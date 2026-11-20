// CEP:FILE: tools/cep_lint/src/main.rs
// CEP:WHAT: CLI entry point for cep_lint: option parsing, exit-code contract, and invocation of the scan-report pipeline.
// CEP:WHY: Standards 50.1 defines the invocation surface (--config/--self-test/--help/--version) and the exit-code contract (0 clean, 1 findings, 2 bootstrap, 3 usage); the binary is the only place that maps outcomes to process exit codes.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns exit 3 on usage errors, exit 2 on bootstrap failures (unreadable/invalid configuration or manifest), exit 1 when findings at fail_on severities exist or a self-test scenario fails, exit 0 otherwise.
// CEP:ASSUMES: The configuration lives at .cep/cep_lint.json by default (CEP_LINT_CONFIG environment override); the tool runs from the repository root so relative doc paths resolve.
// CEP:COST: offline tool; deterministic; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p cep_lint; cep_lint --self-test over tools/cep_lint/tests/manifest.json; DOC-SYNC live gate.
// CEP:SECURITY: Reads repository-trusted files only; no network; no secrets.
//! cep_lint: mechanical enforcement of CEP&CC (standards chapter 50).

mod checks;
mod checks_doc;

use cep_lint_config::{config, function, regexlite, report};

use std::process::ExitCode;

const VERSION_LINE: &str = "cep_lint 0.4.0 (CEP&CC 0.4)";

// CEP:WHAT: Parses options, dispatches to help, version, self-test, or the scan pipeline, and maps outcomes to process exit codes.
// CEP:WHY: 50.1 invocation and exit-code contract.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            err.exit_code()
        }
    }
}

/// Usage and bootstrap failures carry their standards-50.1 exit codes.
enum LintError {
    Usage(String),
    Bootstrap(String),
}

impl LintError {
    fn exit_code(&self) -> ExitCode {
        match self {
            LintError::Usage(_) => ExitCode::from(3),
            LintError::Bootstrap(_) => ExitCode::from(2),
        }
    }
}

impl std::fmt::Display for LintError {
    // CEP:WHAT: Renders lint errors as fixed bootstrap/usage strings.
    // CEP:WHY: Exit-code semantics require unambiguous operator output without allocation-heavy formatting.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total over the two variants.
    // CEP:ASSUMES: none.
    // CEP:COST: O(message length); no allocation.
    // CEP:EVIDENCE: cargo test -p cep_lint exit_code_contract.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintError::Usage(message) => write!(f, "CEP-LINT-BOOT-USAGE: {message}"),
            LintError::Bootstrap(message) => write!(f, "CEP-LINT-BOOT: {message}"),
        }
    }
}

// CEP:ID: CS-a44
// CEP:WHAT: Executes one lint invocation end to end and returns the exit code.
// CEP:WHY: 50.1: help, version, self-test, and scan paths share the option parsing and exit-code mapping.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn run(args: &[String]) -> Result<ExitCode, LintError> {
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(ExitCode::SUCCESS);
            }
            "--version" | "-v" => {
                println!("{VERSION_LINE}");
                return Ok(ExitCode::SUCCESS);
            }
            _ => {}
        }
    }
    let self_test = args.iter().any(|arg| arg == "--self-test" || arg == "-t");
    let effective_config = config_from_args(args)?;
    if self_test {
        return run_self_test(&effective_config);
    }
    let targets: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();
    if targets.is_empty() {
        return Err(LintError::Usage(
            "no files or directories given".to_string(),
        ));
    }
    let (config, raw) = config::load(&effective_config).map_err(LintError::Bootstrap)?;
    let target_paths: Vec<String> = targets.iter().map(|target| (*target).clone()).collect();
    let findings = checks::scan_targets(&config, &raw, &target_paths);
    report::print_findings(&findings);
    let failing = findings
        .iter()
        .any(|finding| config.fail_on.contains(&finding.severity));
    if failing {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

// CEP:WHAT: Resolves the effective configuration path from flags, environment, or the default.
// CEP:WHY: 50.1: --config, -c, CEP_LINT_CONFIG, then .cep/cep_lint.json.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn config_from_args(args: &[String]) -> Result<String, LintError> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--config" || args[index] == "-c" {
            return args
                .get(index + 1)
                .cloned()
                .ok_or_else(|| LintError::Usage("--config requires a path argument".to_string()));
        }
        if let Some(rest) = args[index].strip_prefix("--config=") {
            return Ok(rest.to_string());
        }
        if let Some(rest) = args[index].strip_prefix("-c=") {
            return Ok(rest.to_string());
        }
        index += 1;
    }
    match std::env::var("CEP_LINT_CONFIG") {
        Ok(value) => Ok(value),
        Err(_) => Ok(".cep/cep_lint.json".to_string()),
    }
}

// CEP:WHAT: Prints the usage text with the exit-code contract.
// CEP:WHY: 50.1 --help surface.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
fn print_help() {
    println!(
        "Usage: cep_lint [options] <files or directories>\n\
         Options:\n\
         \x20 --config, -c <path>  lint configuration JSON (default .cep/cep_lint.json, env CEP_LINT_CONFIG)\n\
         \x20 --self-test, -t      run the self-test manifest and exit\n\
         \x20 --help, -h           show this help\n\
         \x20 --version, -v        print tool and standard versions\n\
         Exit codes: 0 no failing findings; 1 findings or failed self-test;\n\
         2 bootstrap failure; 3 usage error."
    );
}

// CEP:ID: CS-a25
// CEP:WHAT: Runs the self-test manifest with exact expected finding counts per scenario.
// CEP:WHY: 50.6 item 3: every manifest scenario must match its expected count for the gate to pass.
// CEP:STATUS: complete
// CEP:FAILURE: none for well-formed inputs; bootstrap diagnostics otherwise (exit-code contract upstream).
// CEP:ASSUMES: Runs from the repository root; inputs are repository-trusted UTF-8 text.
// CEP:COST: offline tool; O(lines * rules) scan cost, deterministic.
// CEP:EVIDENCE: cep_lint --self-test (45 scenarios); self-lint gate over this source tree.
/// Resolves a workspace-relative path: CWD first, then the package parent.
// CEP:WHAT: Resolves workspace-relative paths for test and CLI invocations.
// CEP:WHY: cargo test binaries run from the package root; the gate paths must resolve identically from both contexts.
// CEP:STATUS: complete
// CEP:FAILURE: Returns the CWD-relative path unchanged when no alternative exists (the caller reports the miss).
// CEP:ASSUMES: CARGO_MANIFEST_DIR points at tools/cep_lint, whose parent is the workspace root.
// CEP:COST: O(1) filesystem check.
// CEP:EVIDENCE: cargo test -p cep_lint self_test_gate_passes.
pub(crate) fn resolve_workspace_path(relative: &str) -> String {
    if std::path::Path::new(relative).exists() {
        return relative.to_string();
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|parent| parent.parent())
            .map(|workspace| workspace.join(relative));
        if let Some(candidate) = candidate {
            if candidate.exists() {
                return candidate.display().to_string();
            }
        }
    }
    relative.to_string()
}

// CEP:ID: CS-a25
// CEP:WHAT: Runs the self-test manifest with exact expected finding counts per scenario.
// CEP:WHY: Zero failures is the tool's own gate before it judges anything else.
// CEP:STATUS: complete
// CEP:FAILURE: Bootstrap errors exit 2; scenario mismatches exit 1 with per-scenario diagnostics.
// CEP:ASSUMES: Manifest and fixture paths resolve via the workspace resolver.
// CEP:COST: O(scenarios * fixture size).
// CEP:EVIDENCE: cargo test -p cep_lint self_test_gate_passes.
fn run_self_test(effective_config: &str) -> Result<ExitCode, LintError> {
    // Test binaries run from the package root; resolve workspace-relative
    // paths through CARGO_MANIFEST_DIR when the CWD-relative path misses.
    let manifest_path = resolve_workspace_path("tools/cep_lint/tests/manifest.json");
    let raw = std::fs::read_to_string(&manifest_path).map_err(|err| {
        LintError::Bootstrap(format!("manifest unreadable {manifest_path}: {err}"))
    })?;
    let manifest = config::Json::parse(&raw)
        .map_err(|err| LintError::Bootstrap(format!("manifest parse: {err}")))?;
    let scenarios = manifest
        .get("scenarios")
        .and_then(config::Json::as_array)
        .ok_or_else(|| LintError::Bootstrap("manifest missing scenarios array".to_string()))?;
    let mut failures = 0usize;
    let mut executed = 0usize;
    for scenario in scenarios {
        executed += 1;
        let fixture = scenario
            .get("fixture")
            .and_then(config::Json::as_str)
            .ok_or_else(|| LintError::Bootstrap("scenario missing fixture".to_string()))?;
        let rule = scenario
            .get("rule")
            .and_then(config::Json::as_str)
            .ok_or_else(|| LintError::Bootstrap("scenario missing rule".to_string()))?;
        let expected = scenario
            .get("expected_findings")
            .and_then(config::Json::as_int)
            .ok_or_else(|| {
                LintError::Bootstrap("scenario missing expected_findings".to_string())
            })?;
        let fixture_resolved = resolve_workspace_path(fixture);
        let fixture_text = std::fs::read_to_string(&fixture_resolved).map_err(|err| {
            LintError::Bootstrap(format!("fixture unreadable {fixture_resolved}: {err}"))
        })?;
        let resolved_config = resolve_workspace_path(effective_config);
        let (config, raw) = config::load(&resolved_config).map_err(LintError::Bootstrap)?;
        let actual = checks::count_rule_findings(&config, &raw, fixture, rule, &fixture_text);
        if actual != expected as usize {
            failures += 1;
            println!(
                "[CEP-LINT] SELF-TEST FAIL: {rule} fixture {fixture} expected {expected} findings, found {actual}"
            );
        }
    }
    println!("[CEP-LINT] self-test: {executed} scenarios, {failures} failures");
    if failures == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}
