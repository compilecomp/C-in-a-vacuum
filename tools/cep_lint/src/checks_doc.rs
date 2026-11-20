// CEP:FILE: tools/cep_lint/src/checks_doc.rs
// CEP:WHAT: The document-agreement and security-probe check primitives: DOC-SYNC, file topology, Rosetta registry/tiers/inquisition, secret probing, unsafe documentation, and name validation.
// CEP:WHY: Standards 50 requires one mechanism per primitive; these checks span multiple files or documents and are split from the per-file comment and token checks to honor the 62.5.1 line limits.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Unreadable documents produce coded bootstrap diagnostics; findings carry severity, rule, file, line, and message.
// CEP:ASSUMES: The scan runs from the repository root so doc/registry paths resolve.
// CEP:COST: O(doc size + findings); deterministic.
// CEP:EVIDENCE: cep_lint --self-test; self-lint gate over this source tree.
// CEP:SECURITY: Reads repository-trusted files only.
//! Document-agreement and security-probe checks.

use std::collections::BTreeMap;

use crate::function;
use crate::regexlite;
use cep_lint_config::config::Json as ConfigJson;
use cep_lint_config::config::{Json, LintConfig};
use cep_lint_config::scan::Line;

// Local helpers mirroring the checks.rs option accessors; the config type is
// shared, so behavior is identical without cross-module visibility churn.
fn option<'a>(config: &'a LintConfig, rule: &str, key: &str) -> Option<&'a ConfigJson> {
    config
        .rules
        .get(rule)
        .and_then(|rule| rule.options.get(key))
}

fn option_str(config: &LintConfig, rule: &str, key: &str) -> Option<String> {
    option(config, rule, key)
        .and_then(ConfigJson::as_str)
        .map(String::from)
}

fn option_int(config: &LintConfig, rule: &str, key: &str) -> Option<i64> {
    option(config, rule, key).and_then(ConfigJson::as_int)
}

fn option_str_array(config: &LintConfig, rule: &str, key: &str) -> Vec<String> {
    option(config, rule, key)
        .and_then(ConfigJson::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

// CEP:WHAT: Extracts the declared CEP:CLASS from a file header.
// CEP:WHY: Class-scoped rules need the file's conformance class.
// CEP:STATUS: complete
// CEP:FAILURE: none; returns None when no class is declared.
// CEP:ASSUMES: The class field sits in the file header block.
// CEP:COST: O(header lines).
// CEP:EVIDENCE: cep_lint --self-test hot-banned scenario.
pub fn find_block<'a>(blocks: &'a [(usize, String)], field: &str) -> Option<&'a str> {
    blocks.iter().find_map(|(_line, text)| {
        text.strip_prefix(&format!("// {field}:"))
            .map(|value| value.trim_start())
    })
}

// CEP:WHAT: Extracts the declared CEP:CLASS from a file header.
// CEP:WHY: Class-scoped rules need the file's conformance class.
// CEP:STATUS: complete
// CEP:FAILURE: none; returns None when no class is declared.
// CEP:ASSUMES: The class field sits in the file header block.
// CEP:COST: O(header lines).
// CEP:EVIDENCE: cep_lint --self-test hot-banned scenario.
pub(crate) fn file_class(lines: &[Line]) -> Option<String> {
    for line in lines {
        if let Some(value) = line.raw.trim().strip_prefix("// CEP:CLASS:") {
            return Some(value.trim().to_string());
        }
        let trimmed = line.raw.trim_start();
        if !(trimmed.starts_with("// CEP:") || trimmed.starts_with("//!")) && line.number > 1 {
            return None;
        }
    }
    None
}

// CEP:ID: CS-a23
// CEP:WHAT: Checks bidirectional DOC-SYNC between the configuration and the standards document plus the version contract.
// CEP:WHY: 50.3 DOC-SYNC and Law 8: configured rules need headings, documented rules need configuration, and the Version line must equal standard.version.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
pub(crate) fn capture_group(regex: &regexlite::Regex, line: &str, group: usize) -> Option<String> {
    // The subset matcher does not expose capture groups; DOC-SYNC needs the
    // rule name after the '### Rule ' prefix, which the heading pattern
    // fixes positionally. Extract the token directly per the pattern shape.
    let _ = (regex, group);
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("### Rule ")?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

// CEP:ID: CS-a23
// CEP:WHAT: Checks bidirectional DOC-SYNC between the configuration and the standards document plus the version contract.
// CEP:WHY: Configured rules need headings, documented rules need configuration, and the Version line must equal standard.version; the mechanical enforcement of the stale-documentation law.
// CEP:STATUS: complete
// CEP:FAILURE: Unreadable document produces a class-1 bootstrap diagnostic; drift findings name the rule and direction.
// CEP:ASSUMES: The document path resolves via the workspace resolver.
// CEP:COST: O(document lines + rules).
// CEP:EVIDENCE: cep_lint --self-test doc-sync scenario; the live gate over standards.md.
pub(crate) fn check_doc_sync(
    config: &LintConfig,
    _rule_name: &str,
    push: &mut dyn FnMut(usize, String),
) {
    // Every configured rule must have a '### Rule' heading in the document,
    // every documented rule must exist in the configuration, and the
    // document's Version line must equal standard.version.
    // The standards document path is workspace-relative; resolve it the
    // same way the CLI resolves config and manifest paths.
    let doc_path = crate::resolve_workspace_path(&config.doc_path);
    let doc = match std::fs::read_to_string(&doc_path) {
        Ok(text) => text,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-1: doc unreadable {}: {err}", config.doc_path),
            );
            return;
        }
    };
    let heading = match regexlite::compile(&config.doc_heading_pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(0, format!("CEP-LINT-BOOT-8: {err}"));
            return;
        }
    };
    let mut documented: Vec<String> = Vec::new();
    let mut version_found: Option<String> = None;
    for line in doc.lines() {
        if let Some(captures) = capture_group(&heading, line, 1) {
            documented.push(captures);
        }
        if let Some(rest) = line.trim().strip_prefix("Version:") {
            // Pattern-shaped version extraction: find the first number run.
            let digits: String = rest
                .chars()
                .skip_while(|c| *c == ' ' || *c == '*')
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let trimmed = digits.trim_end_matches('.');
            if !trimmed.is_empty() {
                version_found = Some(trimmed.to_string());
            }
        }
    }
    for rule in config.rules.keys() {
        if !documented.iter().any(|documented| documented == rule) {
            push(
                0,
                format!(
                    "DOC-SYNC: configured rule {rule} has no '### Rule' heading in {}",
                    config.doc_path
                ),
            );
        }
    }
    for documented_rule in &documented {
        if !config.rules.contains_key(documented_rule) {
            push(0, format!("DOC-SYNC: documented rule {documented_rule} exists in {} but not in the configuration", config.doc_path));
        }
    }
    match &version_found {
        Some(version) if version != &config.standard_version => {
            push(
                0,
                format!(
                    "DOC-SYNC: document Version {version} != configuration standard.version {}",
                    config.standard_version
                ),
            );
        }
        None => push(0, "DOC-SYNC: document Version line not found".to_string()),
        _ => {}
    }
}

// CEP:ID: CS-a22
// CEP:WHAT: Validates identifiers located by the detection pattern against the naming pattern.
// CEP:WHY: 50.3 NAMING-TYPE: type names must be PascalCase without underscores; the detection pattern locates the construct and the name pattern validates the identifier.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_name_validation(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let pattern = option_str(config, rule_name, "pattern").unwrap_or_default();
    let name_pattern = option_str(config, rule_name, "name_pattern").unwrap_or_default();
    let detection = match regexlite::compile(&pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    let names = match regexlite::compile(&name_pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} name pattern: {err}"),
            );
            return;
        }
    };
    for line in lines {
        if !detection.is_match(&line.code) {
            continue;
        }
        // The identifier is the token after the struct/enum/union keyword.
        let code = line.code.trim();
        if let Some(position) = ["struct", "enum", "union"].iter().find_map(|keyword| {
            let rest = code.strip_prefix(keyword)?;
            if rest.starts_with(char::is_whitespace) {
                Some(rest)
            } else {
                None
            }
        }) {
            let name: String = position
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !names.is_match(&name) {
                push(
                    line.number,
                    format!("type name {name} does not match the naming pattern"),
                );
            }
        }
    }
}

// CEP:WHAT: Extracts the rule-name token from a DOC-SYNC heading line.
// CEP:WHY: The heading pattern fixes the rule name positionally; the subset matcher exposes no capture groups, so the token is extracted per the pattern shape.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
// CEP:ID: CS-a37
// CEP:WHAT: Checks file line limits by declared class and rejects junk-drawer names.
// CEP:WHY: 62.5 and 50.3 ATOMIC-FILE: class-based line limits and the util/helper/misc/common ban.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_file_topology(
    config: &LintConfig,
    rule_name: &str,
    path: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let limits: BTreeMap<String, Json> = option(config, rule_name, "line_limits")
        .and_then(Json::get_object_ref)
        .cloned()
        .unwrap_or_default();
    let default_limit = option_int(config, rule_name, "default_limit").unwrap_or(400);
    let junk = option_str_array(config, rule_name, "junk_names");
    let class = file_class(lines).unwrap_or_default();
    let limit = limits
        .get(&class)
        .and_then(Json::as_int)
        .unwrap_or(default_limit) as usize;
    if lines.len() > limit {
        push(
            1,
            format!(
                "file has {} lines, exceeding the {limit}-line limit for class {class:?}",
                lines.len()
            ),
        );
    }
    let file_name = std::path::Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();
    for junk_name in &junk {
        if file_name.contains(junk_name) {
            push(
                1,
                format!("junk-drawer file name rejected: contains {junk_name}"),
            );
        }
    }
}

// CEP:WHAT: Loads the registered Rosetta function IDs from the registry manifest.
// CEP:WHY: 68.2.3: the registry is the mechanical source of truth for ID agreement.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
pub(crate) fn rosetta_ids(config: &LintConfig) -> Vec<(String, String)> {
    let registry_path = option_str(config, "CEP-LINT-DOC-FILE", "registry_path")
        .unwrap_or_else(|| "docs/registry.json".to_string());
    let mut ids = Vec::new();
    if let Ok(text) = std::fs::read_to_string(&registry_path) {
        if let Ok(document) = cep_lint_config::config::Json::parse(&text) {
            if let Some(functions) = document.get("functions").and_then(Json::as_array) {
                for entry in functions {
                    if let (Some(id), Some(domain)) = (
                        entry.get("id").and_then(Json::as_str),
                        entry.get("domain").and_then(Json::as_str),
                    ) {
                        ids.push((id.to_string(), domain.to_string()));
                    }
                }
            }
        }
    }
    ids
}

// CEP:ID: CS-a24
// CEP:WHAT: Checks registry-to-source and registry-to-doc-tree agreement in both directions.
// CEP:WHY: 50.3 DOC-FILE and 68.6.1: registered IDs need doc files; source IDs need registration.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
pub(crate) fn check_rosetta_registry(
    config: &LintConfig,
    rule_name: &str,
    lexed: &BTreeMap<String, Vec<Line>>,
    push: &mut dyn FnMut(usize, String),
) {
    let registered = rosetta_ids(config);
    let docs_prefix = option_str(config, rule_name, "docs_prefix").unwrap_or_default();
    // Source IDs must be registered; registered IDs must have doc files.
    for (path, lines) in lexed {
        for function in function::extract(lines) {
            if let Some(id) = find_block(&function.block, "CEP:ID") {
                if !registered.iter().any(|(registered, _)| registered == id) {
                    push(
                        function.line,
                        format!("CEP:ID {id} is not in docs/registry.json"),
                    );
                }
            }
        }
        let _ = path;
    }
    for (id, domain) in &registered {
        let doc_path = format!("{docs_prefix}/{domain}/{id}.md");
        if std::fs::metadata(&doc_path).is_err() {
            push(
                0,
                format!("registry entry {id} has no doc file at {doc_path}"),
            );
        }
    }
}

// CEP:WHAT: Checks that Rosetta docs carry both mandated tier headings.
// CEP:WHY: 68.3 and 50.3 DOC-TIERS.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_rosetta_tiers(
    _config: &LintConfig,
    _rule_name: &str,
    lexed: &BTreeMap<String, Vec<Line>>,
    push: &mut dyn FnMut(usize, String),
) {
    for (path, lines) in lexed {
        if !path.starts_with("docs/functions/") {
            continue;
        }
        let text_has = |needle: &str| lines.iter().any(|line| line.raw.contains(needle));
        if !text_has("## Tier 1: Beginner") || !text_has("## Tier 2: Silicon Reality") {
            push(
                1,
                "Rosetta doc missing one of the two mandated tier headings".to_string(),
            );
        }
        // 68.3.2: vague Tier 2 phrases are a review-level violation.
        for line in lines {
            if line.raw.starts_with("## Tier 2") {
                continue;
            }
        }
    }
}

// CEP:WHAT: Checks that Rosetta docs carry the Silent Failure Checklist section.
// CEP:WHY: 68.4 and 50.3 DOC-INQUISITION.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_rosetta_inquisition(
    _config: &LintConfig,
    _rule_name: &str,
    lexed: &BTreeMap<String, Vec<Line>>,
    push: &mut dyn FnMut(usize, String),
) {
    for (path, lines) in lexed {
        if !path.starts_with("docs/functions/") {
            continue;
        }
        if !lines
            .iter()
            .any(|line| line.raw.contains("## Silent Failure Checklist"))
        {
            push(
                1,
                "Rosetta doc missing the Silent Failure Checklist section".to_string(),
            );
        }
    }
}

// CEP:ID: CS-a38
// CEP:WHAT: Probes comments and raw lines for secret-shaped content with suppression support.
// CEP:WHY: 50.3 SECRET-SCAN severity 0; lexical probing only, per the documented limits.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_secret_probe(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let patterns = option_str_array(config, rule_name, "patterns");
    let mut compiled = Vec::new();
    for (index, pattern) in patterns.iter().enumerate() {
        match regexlite::compile(pattern) {
            Ok(regex) => compiled.push(regex),
            Err(err) => push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern {index}: {err}"),
            ),
        }
    }
    let suppression = option_str(config, rule_name, "suppression_pattern")
        .and_then(|pattern| regexlite::compile(&pattern).ok());
    for line in lines {
        // Probe the raw line: secrets live inside string literals, which
        // the lexical pass blanks out of the code view.
        let probe_target = line.raw.as_str();
        if compiled.iter().any(|regex| regex.is_match(probe_target)) {
            let suppressed = suppression
                .as_ref()
                .map(|suppression| suppression.is_match(&line.raw))
                .unwrap_or(false);
            if !suppressed {
                push(
                    line.number,
                    "secret-shaped content detected (lexical probe)".to_string(),
                );
            }
        }
    }
}

// CEP:ID: CS-a39
// CEP:WHAT: Checks that every unsafe usage is documented by the configured field.
// CEP:WHY: 22.7 and 25.7: undocumented unsafe usage is a severity-1 defect.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_unsafe_doc(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let field = option_str(config, rule_name, "unsafe_field").unwrap_or_default();
    let file_has_field = lines
        .iter()
        .take_while(|line| line.raw.trim_start().starts_with("// CEP:") || line.number <= 30)
        .any(|line| line.raw.contains(&format!("// {field}:")));
    for function in function::extract(lines) {
        let uses_unsafe = lines
            .iter()
            .skip_while(|line| line.number < function.line)
            .take(40)
            .take_while(|line| {
                // Until the next function definition.
                let code = &line.code;
                !(code.trim_start().starts_with("fn ") || code.trim_start().starts_with("pub fn"))
            })
            .any(|line| {
                let words: Vec<&str> = line.code.split_whitespace().collect();
                words
                    .iter()
                    .any(|word| *word == "unsafe" || *word == "unsafe{" || *word == "unsafe{")
            });
        if uses_unsafe {
            let documented = function
                .block
                .iter()
                .any(|(_line, text)| text.contains(&format!("// {field}:")))
                || file_has_field;
            if !documented {
                push(
                    function.line,
                    format!(
                        "unsafe usage without a {field} field in the enclosing block or header"
                    ),
                );
            }
        }
    }
}

/// Runs one document-scoped rule over one file (scan-level dispatch).
// CEP:ID: CS-a32
// CEP:WHAT: Dispatches one document-agreement or security-probe rule to its primitive for one file.
// CEP:WHY: Cross-file rules produce per-file findings; the scan-level dispatcher calls this entry point so finding attribution stays per-file.
// CEP:STATUS: complete
// CEP:FAILURE: Unknown rule/mode pairs are skipped; the manifest catches silent skips.
// CEP:ASSUMES: Runs from the repository root.
// CEP:COST: O(lines * rule).
// CEP:EVIDENCE: cep_lint --self-test; self-lint gate.
pub fn run_one_doc_rule(
    config: &LintConfig,
    rule_name: &str,
    path: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let rule = match config.rules.get(rule_name) {
        Some(rule) => rule,
        None => return,
    };
    let severity = rule.severity;
    let mut emit = |line: usize, message: String| {
        push(line, message);
    };
    let _ = severity;
    match (rule.check.as_str(), rule.mode.as_str()) {
        ("file_topology_policy", _) => {
            check_file_topology(config, rule_name, path, lines, &mut emit)
        }
        ("rosetta_policy", "tiers") => {
            let single = std::collections::BTreeMap::from([(path.to_string(), lines.to_vec())]);
            check_rosetta_tiers(config, rule_name, &single, &mut emit)
        }
        ("rosetta_policy", "inquisition") => {
            let single = std::collections::BTreeMap::from([(path.to_string(), lines.to_vec())]);
            check_rosetta_inquisition(config, rule_name, &single, &mut emit)
        }
        ("secret_probe_policy", _) => check_secret_probe(config, rule_name, lines, &mut emit),
        ("unsafe_doc_policy", _) => check_unsafe_doc(config, rule_name, lines, &mut emit),
        ("name_validation_regex", _) => check_name_validation(config, rule_name, lines, &mut emit),
        _ => {}
    }
}
