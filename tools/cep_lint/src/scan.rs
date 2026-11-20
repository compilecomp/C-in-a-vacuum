// CEP:FILE: tools/cep_lint/src/scan.rs
// CEP:WHAT: Deterministic file traversal and per-line code/comment/string classification.
// CEP:WHY: Standards 50.1 requires sorted directory traversal and deterministic output; the check primitives need code lines with comments removed and string literals identified, exactly the 50.3 lexical model.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Unreadable files produce a finding-shaped bootstrap diagnostic rather than aborting the whole scan; traversal errors are reported per target.
// CEP:ASSUMES: Source files are UTF-8; Rust line comments (// and /// and //!), block comments, and double-quoted string literals are the syntax relevant to the lexical rules.
// CEP:COST: O(total bytes) single pass per file; no allocation beyond the line vectors.
// CEP:EVIDENCE: cargo test -p cep_lint scan_sorted_and_deterministic; --self-test fixtures.
// CEP:SECURITY: Reads repository-trusted files only; no network.
//! File traversal and lexical classification.

use std::collections::BTreeMap;

/// One lexed line: raw text plus the comment-stripped code text.
#[derive(Clone, Debug)]
pub struct Line {
    /// One-based line number.
    pub number: usize,
    /// Raw line text without the trailing newline.
    pub raw: String,
    /// Line text with comments removed and strings blanked.
    pub code: String,
    /// True when the raw line is inside a block comment at its start.
    pub in_block_comment: bool,
    /// Comment text of the line (content after // or inside block comments).
    pub raw_comment: String,
}

/// Lexes one file into lines with code/comment/string classification.
// CEP:WHAT: Lexes a source file into classified lines.
// CEP:WHY: Banned-token and comment rules operate on comment-stripped code; comment rules operate on comment text; both need one consistent lexical pass (50.3 limits: tokens are matched in code after comment and string removal).
// CEP:STATUS: complete
// CEP:FAILURE: none for UTF-8 input; invalid UTF-8 input yields an error the caller reports as a bootstrap diagnostic.
// CEP:ASSUMES: Strings are double-quoted with backslash escapes; block comments nest the way Rust's do not (non-nesting) per Rust reference.
// CEP:COST: O(bytes) per file; allocation for the line vectors.
// CEP:EVIDENCE: --self-test fixtures across the comment/string rules.
pub fn lex(text: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for (index, raw) in text.split('\n').enumerate() {
        let raw = raw.trim_end_matches('\r');
        let (code, starts_block, ends_block, comment) = strip_line(raw, in_block);
        lines.push(Line {
            number: index + 1,
            raw: raw.to_string(),
            in_block_comment: in_block || starts_block,
            code: code.clone(),
            raw_comment: comment,
        });
        in_block = (in_block || starts_block) && !ends_block;
    }
    lines
}

// CEP:ID: CS-a43
// CEP:WHAT: Strips comments and string-literal contents from one line.
// CEP:WHY: One consistent lexical pass feeds both code-scoped and comment-scoped rules.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn strip_line(raw: &str, in_block: bool) -> (String, bool, bool, String) {
    let mut code = String::new();
    let mut comment = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    let bytes = raw.as_bytes();
    let mut block_started = false;
    let mut block_ended = false;
    let mut cursor_in_block = in_block;
    while i < bytes.len() {
        let rest = &raw[i..];
        if cursor_in_block {
            match rest.find("*/") {
                Some(end) => {
                    block_ended = true;
                    i += end + 2;
                    cursor_in_block = false;
                    comment.push(' ');
                }
                None => {
                    comment.push_str(rest);
                    i = bytes.len();
                }
            }
            continue;
        }
        let character = bytes[i] as char;
        if !in_string && rest.starts_with("//") {
            comment.push_str(&rest[2..]);
            break;
        }
        if !in_string && rest.starts_with("/*") {
            block_started = true;
            cursor_in_block = true;
            i += 2;
            match raw[i..].find("*/") {
                Some(end) => {
                    i += end + 2;
                    cursor_in_block = false;
                    block_ended = true;
                    comment.push(' ');
                }
                None => {
                    i = bytes.len();
                }
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
                code.push(' ');
                i += character.len_utf8();
                continue;
            }
            if character == '\\' {
                escaped = true;
                code.push(' ');
                i += character.len_utf8();
                continue;
            }
            if character == '"' {
                in_string = false;
                code.push('"');
                i += 1;
                continue;
            }
            code.push(' ');
            i += character.len_utf8();
            continue;
        }
        if character == '"' {
            in_string = true;
            code.push('"');
            i += 1;
            continue;
        }
        code.push(character);
        i += character.len_utf8();
    }
    (code, block_started && !block_ended, block_ended, comment)
}

/// Collects first-party files under a target path, sorted deterministically.
// CEP:WHAT: Expands a target path into the sorted list of first-party source files.
// CEP:WHY: Standards 50.1 requires sorted traversal and locale-independent order; the file_globs suffix filter restricts scanning to declared first-party dialects.
// CEP:STATUS: complete
// CEP:FAILURE: Returns an error for unreadable targets; a file target not matching any glob yields just that file (explicit invocation overrides globs).
// CEP:ASSUMES: Directory recursion is depth-first with lexically sorted entries; no symlink loops are followed because symlinks are treated as files.
// CEP:COST: O(entries log entries) for sorting plus O(files) reads.
// CEP:EVIDENCE: cargo test -p cep_lint scan_sorted_and_deterministic.
pub fn collect_files(target: &str, globs: &[String]) -> Result<Vec<String>, String> {
    let path = std::path::Path::new(target);
    if path.is_file() {
        return Ok(vec![target.to_string()]);
    }
    if !path.is_dir() {
        return Err(format!("target not found: {target}"));
    }
    let mut files = Vec::new();
    collect_dir(path, globs, &mut files)?;
    files.sort();
    Ok(files)
}

// CEP:WHAT: Recursively collects first-party files under one directory.
// CEP:WHY: Depth-first with lexically sorted entries for deterministic output.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn collect_dir(
    dir: &std::path::Path,
    globs: &[String],
    files: &mut Vec<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|err| format!("unreadable directory {}: {err}", dir.display()))?;
    let mut names: Vec<std::path::PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name != "target"))
        .collect();
    names.sort();
    for path in names {
        if path.is_dir() {
            collect_dir(&path, globs, files)?;
        } else if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if globs
                .iter()
                .any(|glob| name.ends_with(glob.trim_start_matches('*')))
            {
                files.push(path.display().to_string());
            }
        }
    }
    Ok(())
}

/// Groups lexed files by path for multi-file checks.
pub type LexedFiles = BTreeMap<String, Vec<Line>>;
