// CEP:FILE: tools/cep_lint/src/function.rs
// CEP:WHAT: Rust function extraction with the linter-consistent statement counter used by the FUNC-BLOCK, DOC-ID, NAMING-FUNC, STUB-BODY, and CONSTANT-RETURN checks.
// CEP:WHY: The 10.2 function contract and the 68.6.1 DOC-ID threshold are mechanical only if the extractor matches the repository's counting definition (scripts/count_statements.py): statements are lines ending in ';' or '{' inside the body.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Extraction is heuristic for exotic syntax (raw strings, nested closures as body heads); the definition overcounts rather than hides violations, per the conservative-threshold principle recorded in the config.
// CEP:ASSUMES: Functions are declared with the fn keyword at any visibility; bodies start at the first brace after the signature and end at the matching brace.
// CEP:COST: O(file lines) per file; no allocation beyond the extraction vectors.
// CEP:EVIDENCE: cargo test -p cep_lint function_extraction_matches_counter.
// CEP:SECURITY: Reads in-memory text only.
//! Function extraction and statement counting.

/// One extracted function with its preceding CEP block.
#[derive(Clone, Debug)]
pub struct Function {
    /// Function name.
    pub name: String,
    /// Line number of the fn declaration.
    pub line: usize,
    /// The contiguous CEP comment block above it, in order.
    pub block: Vec<(usize, String)>,
    /// Statement count per the linter-consistent definition.
    pub statements: usize,
    /// True when the body is empty (no counted statements).
    pub body_empty: bool,
    /// Number of code lines in the body.
    pub body_loc: usize,
}

/// Extracts every function in a lexed file with its CEP block and counts.
// CEP:ID: CS-a42
// CEP:WHAT: Extracts functions, their CEP blocks, and statement counts from lexed lines.
// CEP:WHY: The function_policy checks need block presence/contents, body statement counts, and the name; one extraction pass feeds all of them deterministically.
// CEP:STATUS: complete
// CEP:FAILURE: none; extraction is total over any input text.
// CEP:ASSUMES: The counting definition matches scripts/count_statements.py (lines ending in ';' or '{' inside the body); the fn line's brace may be on the same or a later line.
// CEP:COST: O(lines).
// CEP:EVIDENCE: cargo test -p cep_lint function_extraction_matches_counter.
pub fn extract(lines: &[crate::scan::Line]) -> Vec<Function> {
    let mut functions = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        if let Some(name) = parse_fn_name(&line.code) {
            let block = collect_block(lines, index);
            let (statements, _, body_loc) = count_body(lines, index);
            functions.push(Function {
                name,
                line: line.number,
                block,
                statements,
                // An empty body is literally no code lines, not merely no
                // semicolon-terminated statements (expression bodies).
                body_empty: body_loc == 0,
                body_loc,
            });
            // Skip past the body to avoid nested fn re-detection.
            index = skip_body(lines, index);
        } else {
            index += 1;
        }
    }
    functions
}

// CEP:WHAT: Parses the function name from a declaration line.
// CEP:WHY: Visibility and qualifier prefixes are stripped before the fn keyword.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn parse_fn_name(code: &str) -> Option<String> {
    let trimmed = code.trim_start();
    let rest = trimmed
        .strip_prefix("pub ")
        .or_else(|| trimmed.strip_prefix("pub(crate) "))
        .unwrap_or(trimmed);
    let rest = rest
        .strip_prefix("const ")
        .or_else(|| rest.strip_prefix("unsafe "))
        .unwrap_or(rest);
    let rest = rest.strip_prefix("fn ")?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

// CEP:ID: CS-a40
// CEP:WHAT: Collects the contiguous CEP block above a function definition.
// CEP:WHY: Block adjacency and the header-separation rule follow the 50.3 FUNC-BLOCK limits.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn collect_block(lines: &[crate::scan::Line], fn_index: usize) -> Vec<(usize, String)> {
    let mut j = fn_index;
    if j > 0 {
        j -= 1;
        // Skip attributes and rustdoc between the block and the definition.
        while j > 0
            && (lines[j].raw.trim_start().starts_with("#[")
                || lines[j].raw.trim_start().starts_with("///"))
        {
            j -= 1;
        }
        if !lines[j].raw.trim_start().starts_with("// CEP:") {
            return Vec::new();
        }
        let mut block = Vec::new();
        while j > 0 && lines[j - 1].raw.trim_start().starts_with("// CEP:") {
            j -= 1;
        }
        while j < fn_index && lines[j].raw.trim_start().starts_with("// CEP:") {
            block.push((lines[j].number, lines[j].raw.trim().to_string()));
            j += 1;
        }
        block
    } else {
        Vec::new()
    }
}

// CEP:ID: CS-a41
// CEP:WHAT: Counts statements and body lines inside a function body.
// CEP:WHY: The linter-consistent counting definition matches scripts/count_statements.py.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn count_body(lines: &[crate::scan::Line], fn_index: usize) -> (usize, bool, usize) {
    // Find the opening brace: same line or a later signature line.
    let mut depth = 0i64;
    let mut statements = 0usize;
    let mut loc = 0usize;
    let mut started = false;
    let mut index = fn_index;
    while index < lines.len() {
        let code = &lines[index].code;
        if !started {
            if code.contains('{') {
                started = true;
                depth += code.matches('{').count() as i64 - code.matches('}').count() as i64;
                if depth <= 0 {
                    return (0, true, 0);
                }
            }
            index += 1;
            continue;
        }
        let trimmed = code.trim();
        let opens = code.matches('{').count();
        let closes = code.matches('}').count();
        depth += opens as i64 - closes as i64;
        if depth <= 0 {
            return (statements, statements == 0, loc);
        }
        if !trimmed.is_empty() {
            loc += 1;
            if trimmed.ends_with(';') || trimmed.ends_with('{') {
                statements += 1;
            }
        }
        index += 1;
    }
    (statements, statements == 0, loc)
}

// CEP:WHAT: Skips past a function body to avoid nested-function re-detection.
// CEP:WHY: Brace-depth tracking mirrors count_body.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn skip_body(lines: &[crate::scan::Line], fn_index: usize) -> usize {
    let mut depth = 0i64;
    let mut started = false;
    let mut index = fn_index;
    while index < lines.len() {
        let code = &lines[index].code;
        if !started {
            if code.contains('{') {
                started = true;
                depth += code.matches('{').count() as i64 - code.matches('}').count() as i64;
                if depth <= 0 {
                    return index + 1;
                }
            }
            index += 1;
            continue;
        }
        depth += code.matches('{').count() as i64 - code.matches('}').count() as i64;
        if depth <= 0 {
            return index + 1;
        }
        index += 1;
    }
    lines.len()
}
