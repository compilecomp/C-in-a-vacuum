// CEP:FILE: tools/cep_lint/src/checks.rs
// CEP:WHAT: The check primitives implementing the 25 configured rules: file and function comment schemas, lexical policies, topology limits, Rosetta agreement, DOC-SYNC, secret probing, and unsafe documentation.
// CEP:WHY: Standards 50 requires the engine to contain mechanism only while every threshold and pattern lives in the configuration; each primitive is the mechanism named in the config's check field.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: A rule whose primitive cannot execute (bad regex, missing doc file) yields a bootstrap diagnostic; findings carry severity, rule, file, line, and message.
// CEP:ASSUMES: The scan runs from the repository root; first-party files are Rust sources matching the configured globs.
// CEP:COST: O(files * lines + rules * patterns) with compiled patterns per file scan.
// CEP:EVIDENCE: cep_lint --self-test (50 scenarios, exact expected counts per rule); self-lint gate over this source tree.
// CEP:SECURITY: Reads repository-trusted files only.
//! Check primitives for the configured rules.

use std::collections::BTreeMap;

use crate::function;
use crate::regexlite;
use cep_lint_config::config::{Json, LintConfig};
use cep_lint_config::report::Finding;
use cep_lint_config::scan::{self, Line};

/// Scans targets and returns all findings, sorted deterministically.
// CEP:ID: CS-a16
// CEP:WHAT: Runs every configured rule over every first-party file under the targets.
// CEP:WHY: One scan pass collects findings across all rules; deterministic sorting (file, line, rule, message) is the 50.1 output contract.
// CEP:STATUS: complete
// CEP:FAILURE: Unreadable targets produce a severity-1 CEP-LINT-BOOT finding rather than aborting the remaining scan.
// CEP:ASSUMES: The tool runs from the repository root so doc/registry paths resolve.
// CEP:COST: O(files * rules * lines).
// CEP:EVIDENCE: cep_lint --self-test.
pub fn scan_targets(config: &LintConfig, raw: &Json, targets: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut lexed: BTreeMap<String, Vec<Line>> = BTreeMap::new();
    for target in targets {
        match scan::collect_files(target, &config.file_globs) {
            Ok(files) => {
                for file in files {
                    match std::fs::read_to_string(&file) {
                        Ok(text) => {
                            lexed.insert(file.clone(), scan::lex(&text));
                        }
                        Err(err) => findings.push(Finding {
                            severity: 1,
                            rule: "CEP-LINT-BOOT".to_string(),
                            file: file.clone(),
                            line: 0,
                            message: format!("unreadable file: {err}"),
                        }),
                    }
                }
            }
            Err(err) => findings.push(Finding {
                severity: 1,
                rule: "CEP-LINT-BOOT".to_string(),
                file: target.clone(),
                line: 0,
                message: err,
            }),
        }
    }
    // Scan-level rules (DOC-SYNC, deferred infrastructure rules) run once
    // per scan, not once per file.
    run_scan_level_rules(config, raw, &mut findings);
    for (path, lines) in &lexed {
        run_file_rules(config, raw, path, lines, &lexed, &mut findings);
    }
    findings.sort_by(|a, b| {
        (&a.file, a.line, &a.rule, &a.message).cmp(&(&b.file, b.line, &b.rule, &b.message))
    });
    findings
}

/// Counts findings for one rule over one in-memory fixture (self-test path).
// CEP:ID: CS-b01
// CEP:WHAT: Runs a single rule against a fixture and counts its findings.
// CEP:WHY: The 50.6 self-verification gate requires exact expected finding counts per rule; the self-test runner needs a single-rule entry point.
// CEP:STATUS: complete
// CEP:FAILURE: Unknown rule names return zero findings, which surfaces as a self-test failure against the manifest's expectation.
// CEP:ASSUMES: The fixture path is used for header/topology rules that need a filename.
// CEP:COST: O(fixture lines * rule).
// CEP:EVIDENCE: cep_lint --self-test.
pub fn count_rule_findings(
    config: &LintConfig,
    raw: &Json,
    fixture: &str,
    rule: &str,
    text: &str,
) -> usize {
    let lines = scan::lex(text);
    let mut findings = Vec::new();
    let single: BTreeMap<String, Vec<Line>> =
        BTreeMap::from([(fixture.to_string(), lines.clone())]);
    run_one_rule(
        config,
        raw,
        rule,
        fixture,
        &lines,
        &single,
        &single,
        &mut findings,
    );
    let rule_body = config.rules.get(rule);
    let moved = rule_body
        .map(|rule_body| {
            matches!(
                rule_body.check.as_str(),
                "file_topology_policy"
                    | "rosetta_policy"
                    | "secret_probe_policy"
                    | "unsafe_doc_policy"
                    | "name_validation_regex"
            )
        })
        .unwrap_or(false);
    if moved {
        crate::checks_doc::run_one_doc_rule(config, rule, fixture, &lines, &mut |line, message| {
            let severity = rule_body.map(|rule_body| rule_body.severity).unwrap_or(1);
            findings.push(Finding {
                severity,
                rule: rule.to_string(),
                file: fixture.to_string(),
                line,
                message,
            });
        });
    }
    findings.len()
}

fn severity_of(config: &LintConfig, rule: &str) -> i64 {
    config
        .rules
        .get(rule)
        .map(|rule| rule.severity)
        .unwrap_or(1)
}

fn option<'a>(config: &'a LintConfig, rule: &str, key: &str) -> Option<&'a Json> {
    config
        .rules
        .get(rule)
        .and_then(|rule| rule.options.get(key))
}

fn option_str(config: &LintConfig, rule: &str, key: &str) -> Option<String> {
    option(config, rule, key)
        .and_then(Json::as_str)
        .map(String::from)
}

fn option_int(config: &LintConfig, rule: &str, key: &str) -> Option<i64> {
    option(config, rule, key).and_then(Json::as_int)
}

fn option_str_array(config: &LintConfig, rule: &str, key: &str) -> Vec<String> {
    option(config, rule, key)
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

// CEP:ID: CS-a17
// CEP:WHAT: Runs scan-level rules once per scan: DOC-SYNC and the deferred receipt-infrastructure rules.
// CEP:WHY: DOC-SYNC compares the configuration against the standards document and must run once, not per file; the three receipt rules emit recorded-deviation findings per their tickets.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn run_scan_level_rules(config: &LintConfig, _raw: &Json, findings: &mut Vec<Finding>) {
    for (rule_name, rule) in &config.rules {
        let severity = rule.severity;
        let mut push = |line: usize, message: String| {
            findings.push(Finding {
                severity,
                rule: rule_name.to_string(),
                file: rule_name.to_string(),
                line,
                message,
            });
        };
        match rule.check.as_str() {
            "commit_hunt_manifest" => push(
                0,
                "hunt-receipt infrastructure deferred per ticket CEP-90 (recorded deviation; no signed receipts exist)".to_string(),
            ),
            "benchmark_manifest" => push(
                0,
                "benchmark-receipt infrastructure deferred per ticket CEP-91 (recorded deviation; no PMC hardware contract exists)".to_string(),
            ),
            "hunter_receipt" => push(
                0,
                "HUNTER receipt infrastructure deferred per ticket CEP-90 (recorded deviation)".to_string(),
            ),
            _ => {}
        }
    }
}

// CEP:WHAT: Runs every file-scoped configured rule over one lexed file.
// CEP:WHY: Rule dispatch iterates the configuration so the engine stays data-driven (50: mechanism only).
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn run_file_rules(
    config: &LintConfig,
    raw: &Json,
    path: &str,
    lines: &[Line],
    lexed: &BTreeMap<String, Vec<Line>>,
    findings: &mut Vec<Finding>,
) {
    for rule_name in config.rules.keys() {
        run_one_rule(config, raw, rule_name, path, lines, lexed, lexed, findings);
    }
}

// CEP:ID: CS-a18
// CEP:WHAT: Dispatches one rule to its configured check primitive with severity and options bound.
// CEP:WHY: The single dispatch point keeps primitive signatures uniform and makes unknown rule/mode pairs silently skipped, which the manifest would catch.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
#[allow(clippy::too_many_arguments)]
fn run_one_rule(
    config: &LintConfig,
    _raw: &Json,
    rule_name: &str,
    path: &str,
    lines: &[Line],
    _single: &BTreeMap<String, Vec<Line>>,
    lexed: &BTreeMap<String, Vec<Line>>,
    findings: &mut Vec<Finding>,
) {
    let severity = severity_of(config, rule_name);
    let rule = match config.rules.get(rule_name) {
        Some(rule) => rule,
        None => return,
    };
    let mut push = |line: usize, message: String| {
        findings.push(Finding {
            severity,
            rule: rule_name.to_string(),
            file: path.to_string(),
            line,
            message,
        });
    };

    match (rule.check.as_str(), rule.mode.as_str()) {
        ("comment_block_schema", "file_header") => {
            check_file_header(config, rule_name, path, lines, &mut push);
        }
        ("comment_block_schema", "status_value") => {
            check_status_value(config, rule_name, lines, &mut push);
        }
        ("comment_block_schema", "stub_requirements") => {
            check_stub_requirements(config, rule_name, lines, &mut push);
        }
        ("function_policy", "block") => {
            check_function_block(config, rule_name, lines, &mut push);
        }
        ("function_policy", "id") => {
            check_function_id(config, rule_name, lines, &mut push);
        }
        ("function_policy", "naming") => {
            check_function_naming(config, rule_name, lines, &mut push);
        }
        ("function_policy", "stub_body") => {
            check_stub_body(config, rule_name, lines, &mut push);
        }
        ("comment_regex", _) => {
            check_comment_regex(config, rule_name, lines, &mut push);
        }
        ("string_literal_regex", _) => {
            check_string_regex(config, rule_name, lines, &mut push);
        }
        ("numeric_literal_policy", _) => {
            check_numeric_literals(config, rule_name, lines, &mut push);
        }
        ("macro_prefix_policy", _) => {
            check_macro_prefix(config, rule_name, lines, &mut push);
        }
        ("doc_rule_coverage", _) => {
            crate::checks_doc::check_doc_sync(config, rule_name, &mut push);
        }
        ("rosetta_policy", "registry") => {
            crate::checks_doc::check_rosetta_registry(config, rule_name, lexed, &mut push);
        }
        ("banned_token_policy", _) => {
            check_banned_tokens(config, rule_name, path, lines, &mut push);
        }
        ("flat_code_regex", _) => {
            check_flat_code_regex(config, rule_name, lines, &mut push);
        }
        _ => {}
    }
}

fn find_block<'a>(blocks: &'a [(usize, String)], field: &str) -> Option<&'a str> {
    blocks.iter().find_map(|(_line, text)| {
        text.strip_prefix(&format!("// {field}:"))
            .map(|value| value.trim_start())
    })
}

fn block_has(blocks: &[(usize, String)], field: &str) -> bool {
    find_block(blocks, field).is_some()
}

// CEP:WHAT: Extracts the declared CEP:CLASS from a file header.
// CEP:WHY: Class-scoped rules need the file's conformance class.
// CEP:STATUS: complete
// CEP:FAILURE: none; returns None when no class is declared.
// CEP:ASSUMES: The class field sits in the file header block.
// CEP:COST: O(header lines).
// CEP:EVIDENCE: cep_lint --self-test hot-banned scenario.
pub(crate) fn file_class(lines: &[Line]) -> Option<String> {
    // The file header is the first contiguous CEP block; its CLASS field
    // classifies the file for class-scoped rules.
    for line in lines {
        if let Some(value) = line.raw.trim().strip_prefix("// CEP:CLASS:") {
            return Some(value.trim().to_string());
        }
        // The header ends at the first non-CEP, non-rustdoc line.
        let trimmed = line.raw.trim_start();
        if !(trimmed.starts_with("// CEP:") || trimmed.starts_with("//!")) && line.number > 1 {
            return None;
        }
    }
    None
}

// CEP:ID: CS-a19
// CEP:WHAT: Checks the file-header schema: position, required fields, and legal class.
// CEP:WHY: 35 and 50.3 FILE-HEADER: the header must be the first content, carry every required field, and declare an allowed class.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_file_header(
    config: &LintConfig,
    rule_name: &str,
    path: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let required = option_str_array(config, rule_name, "required_fields");
    let max_start = option_int(config, rule_name, "max_start_line").unwrap_or(1) as usize;
    let allowed = option_str_array(config, rule_name, "allowed_classes");
    // Header must begin at max_start and be the first contiguous CEP block.
    let header_line = lines
        .iter()
        .find(|line| line.raw.trim_start().starts_with("// CEP:"));
    let line_number = header_line.map(|line| line.number).unwrap_or(0);
    if line_number > max_start {
        push(line_number.max(1), "file header is not the first content (license/shebang above it is a violation by design)".to_string());
    }
    if header_line.is_none() {
        push(1, "missing CEP file header".to_string());
        return;
    }
    let mut block = Vec::new();
    for line in lines {
        let trimmed = line.raw.trim_start();
        if trimmed.starts_with("// CEP:") {
            block.push((line.number, trimmed.to_string()));
        } else if !block.is_empty() {
            break;
        }
    }
    for field in &required {
        if !block_has(&block, field) {
            push(
                line_number,
                format!("file header missing required field {field}"),
            );
        }
    }
    if let Some(class) = find_block(&block, "CEP:CLASS") {
        if !allowed.iter().any(|allowed| allowed == class) {
            push(
                line_number,
                format!("declared class {class} is not in the allowed set"),
            );
        }
    }
    let _ = path;
}

// CEP:WHAT: Checks every CEP:STATUS value against the allowed set.
// CEP:WHY: 10.5: status values are complete, partial, stub, or placeholder only.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_status_value(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let allowed = option_str_array(config, rule_name, "allowed_values");
    for line in lines {
        if let Some(value) = line.raw.trim().strip_prefix("// CEP:STATUS:") {
            let value = value.trim();
            if !allowed.iter().any(|allowed| allowed == value) {
                push(
                    line.number,
                    format!("CEP:STATUS value {value} is not one of {allowed:?}"),
                );
            }
        }
    }
}

// CEP:WHAT: Checks that stub, partial, and placeholder blocks declare owner and ticket.
// CEP:WHY: 10.10 and STUB-DECL: unowned stubs are banned.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_stub_requirements(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let statuses = option_str_array(config, rule_name, "statuses");
    let required = option_str_array(config, rule_name, "required_fields");
    let functions = function::extract(lines);
    for function in &functions {
        if let Some(status) = find_block(&function.block, "CEP:STATUS") {
            if statuses.iter().any(|candidate| candidate == status) {
                for field in &required {
                    if !block_has(&function.block, field) {
                        push(
                            function.line,
                            format!("{status} block missing required field {field}"),
                        );
                    }
                }
            }
        }
    }
}

// CEP:WHAT: Reports whether only attribute, rustdoc, and signature lines sit between a block and its definition.
// CEP:WHY: 50.3 FUNC-BLOCK limits: only punctuation-free declaration-adjacent lines may separate block and definition.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
// CEP:ID: CS-a20
// CEP:WHAT: Checks every nontrivial function for a complete CEP comment block.
// CEP:WHY: 10.2: functions with at least min_body_statements need the seven-field block.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_function_block(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let required = option_str_array(config, rule_name, "required_fields");
    let min_statements = option_int(config, rule_name, "min_body_statements").unwrap_or(2) as usize;
    let functions = function::extract(lines);
    for function in &functions {
        if function.statements < min_statements {
            continue;
        }
        if function.block.is_empty() {
            push(
                function.line,
                format!("function {} lacks a CEP comment block", function.name),
            );
            continue;
        }
        for field in &required {
            if !block_has(&function.block, field) {
                push(
                    function.line,
                    format!(
                        "function {} block missing required field {field}",
                        function.name
                    ),
                );
            }
        }
    }
}

// CEP:ID: CS-a21
// CEP:WHAT: Checks that functions over the statement threshold declare a registered-format CEP:ID.
// CEP:WHY: 68.6.1 mechanical form: more than max_id_statements statements require a CEP:ID matching the domain pattern.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_function_id(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let max_statements = option_int(config, rule_name, "max_id_statements").unwrap_or(10) as usize;
    let id_pattern = option_str(config, rule_name, "id_pattern").unwrap_or_default();
    let matcher = regexlite::compile(&id_pattern);
    let functions = function::extract(lines);
    for function in &functions {
        if function.statements > max_statements {
            match find_block(&function.block, "CEP:ID") {
                None => push(
                    function.line,
                    format!(
                        "function {} has {} statements (> {max_statements}) but declares no CEP:ID",
                        function.name, function.statements
                    ),
                ),
                Some(id) => match &matcher {
                    Ok(regex) => {
                        if !regex.is_match(id) {
                            push(
                                function.line,
                                format!("CEP:ID {id} does not match the configured id pattern"),
                            );
                        }
                    }
                    Err(err) => push(function.line, format!("CEP-LINT-BOOT-8: {err}")),
                },
            }
        }
    }
}

// CEP:WHAT: Checks function names against the configured naming pattern.
// CEP:WHY: 33.13/33.20: snake_case functions.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_function_naming(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let name_pattern = option_str(config, rule_name, "name_pattern").unwrap_or_default();
    let regex = match regexlite::compile(&name_pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    for function in function::extract(lines) {
        if !regex.is_match(&function.name) {
            push(
                function.line,
                format!(
                    "function name {} does not match the naming pattern",
                    function.name
                ),
            );
        }
    }
}

// CEP:WHAT: Checks that empty-bodied functions declare a sanctioned stub status.
// CEP:WHY: 50.3 STUB-BODY: empty bodies are violations unless the block declares stub, partial, or placeholder.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_stub_body(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let suppress = option_str_array(config, rule_name, "suppress_statuses");
    for function in function::extract(lines) {
        if function.body_empty {
            let declared = find_block(&function.block, "CEP:STATUS")
                .map(|status| suppress.iter().any(|candidate| candidate == status))
                .unwrap_or(false);
            if !declared {
                push(
                    function.line,
                    format!(
                        "function {} has an empty body without a declared stub status",
                        function.name
                    ),
                );
            }
        }
    }
}

// CEP:ID: CS-a33
// CEP:WHAT: Applies the configured pattern to comment text with suppression support.
// CEP:WHY: the comment-marker rules operate on comment text
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_comment_regex(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let pattern = option_str(config, rule_name, "pattern").unwrap_or_default();
    let icase = option(config, rule_name, "icase")
        .and_then(Json::as_bool)
        .unwrap_or(false);
    let suppression = option_str(config, rule_name, "suppression_pattern");
    let source = if icase && !pattern.starts_with("(?i)") {
        format!("(?i){pattern}")
    } else {
        pattern
    };
    let regex = match regexlite::compile(&source) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    let suppressor = suppression
        .as_deref()
        .and_then(|suppression| regexlite::compile(suppression).ok());
    for line in lines {
        let comment = line.raw_comment.trim_start();
        if comment.is_empty() {
            continue;
        }
        if regex.is_match(comment) {
            let suppressed = suppressor
                .as_ref()
                .map(|suppressor| suppressor.is_match(comment))
                .unwrap_or(false);
            if !suppressed {
                push(
                    line.number,
                    format!("comment matches banned pattern: {}", line.raw.trim()),
                );
            }
        }
    }
}

// CEP:ID: CS-a34
// CEP:WHAT: Applies the configured pattern to string-literal contents.
// CEP:WHY: 50.3 PLACEHOLDER-TEXT: placeholders must fail loudly, not leak into runtime messages.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_string_regex(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let pattern = option_str(config, rule_name, "pattern").unwrap_or_default();
    let icase = option(config, rule_name, "icase")
        .and_then(Json::as_bool)
        .unwrap_or(false);
    let source = if icase && !pattern.starts_with("(?i)") {
        format!("(?i){pattern}")
    } else {
        pattern
    };
    let regex = match regexlite::compile(&source) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    {
        for line in lines {
            // String-literal contents: find quoted segments in the raw line.
            let raw = &line.raw;
            let bytes: Vec<char> = raw.chars().collect();
            let mut in_string = false;
            let mut segment = String::new();
            let mut escaped = false;
            for character in bytes {
                if escaped {
                    segment.push(character);
                    escaped = false;
                    continue;
                }
                match character {
                    '\\' if in_string => {
                        escaped = true;
                    }
                    '"' => {
                        if in_string {
                            if regex.is_match(&segment) {
                                push(
                                    line.number,
                                    format!("string literal matches banned pattern: \"{segment}\""),
                                );
                            }
                            segment.clear();
                            in_string = false;
                        } else {
                            in_string = true;
                        }
                    }
                    _ if in_string => segment.push(character),
                    _ => {}
                }
            }
        }
    }
}

// CEP:ID: CS-a35
// CEP:WHAT: Flags numeric literals outside the allowed set without a sanctioned context.
// CEP:WHY: Law 7 and 50.3 MAGIC-NUMBER: literals are allowed in const, static, enum, struct, attribute, and comment contexts per configuration.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_numeric_literals(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let allowed = option(config, rule_name, "allowed_values")
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_int).collect::<Vec<i64>>())
        .unwrap_or_default();
    let exempt = option_str_array(config, rule_name, "exempt_contexts");
    let exempt_patterns: Vec<_> = exempt
        .iter()
        .filter_map(|pattern| regexlite::compile(pattern).ok())
        .collect();
    for line in lines {
        if exempt_patterns
            .iter()
            .any(|pattern| pattern.is_match(&line.code))
        {
            continue;
        }
        // Numeric literals beyond the allow list in code lines.
        for token in line.code.split(|c: char| !c.is_ascii_digit() && c != '_') {
            let token = token.trim_matches('_');
            if token.is_empty() {
                continue;
            }
            if let Ok(value) = token.parse::<i64>() {
                if !allowed.contains(&value) {
                    push(
                        line.number,
                        format!("numeric literal {value} outside the allowed set without a sanctioned context"),
                    );
                }
            }
        }
    }
}

// CEP:ID: CS-a36
// CEP:WHAT: Checks macro_rules! names against the configured prefix pattern.
// CEP:WHY: 50.3 MACRO-PREFIX and the 0.3 applicability note: Rust macros obey the object-like macro naming rules.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_macro_prefix(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let name_pattern = option_str(config, rule_name, "name_pattern").unwrap_or_default();
    let regex = match regexlite::compile(&name_pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    for line in lines {
        if let Some(rest) = line.code.trim().strip_prefix("macro_rules! ") {
            // The macro name is the first whitespace-delimited token; the
            // brace belongs to the body, not the name.
            let name = rest.split_whitespace().next().unwrap_or_default();
            if !regex.is_match(name) {
                push(
                    line.number,
                    format!("macro_rules! {name} does not match the configured macro prefix"),
                );
            }
        }
    }
}

// CEP:WHAT: Checks class-scoped banned tokens in comment-stripped code.
// CEP:WHY: 50.3 HOT-BANNED: tokens are matched in code after comment and string removal.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_banned_tokens(
    config: &LintConfig,
    rule_name: &str,
    path: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let applies = option_str_array(config, rule_name, "applies_to_classes");
    let tokens = option_str_array(config, rule_name, "tokens");
    let class = file_class(lines).unwrap_or_default();
    if !applies.iter().any(|candidate| candidate == &class) {
        return;
    }
    for line in lines {
        for token in &tokens {
            if line.code.contains(token.as_str()) {
                push(
                    line.number,
                    format!("banned token in {class} code: {token}"),
                );
            }
        }
    }
    let _ = path;
}

// CEP:WHAT: Applies a structural violation pattern to comment-stripped code lines.
// CEP:WHY: 50.3 EMPTY-CATCH and EMPTY-BRANCH: structural emptiness checks.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn check_flat_code_regex(
    config: &LintConfig,
    rule_name: &str,
    lines: &[Line],
    push: &mut dyn FnMut(usize, String),
) {
    let pattern = option_str(config, rule_name, "pattern").unwrap_or_default();
    let regex = match regexlite::compile(&pattern) {
        Ok(regex) => regex,
        Err(err) => {
            push(
                0,
                format!("CEP-LINT-BOOT-8: rule {rule_name} pattern: {err}"),
            );
            return;
        }
    };
    for line in lines {
        if regex.is_match(&line.code) {
            push(
                line.number,
                "code line matches a structural violation pattern".to_string(),
            );
        }
    }
}
