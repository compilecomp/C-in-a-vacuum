// CEP:FILE: tools/cep_lint/src/report.rs
// CEP:WHAT: Finding collection and the deterministic report printer.
// CEP:WHY: Standards 50.1 requires issues sorted by file, line, rule, and message with severity-class output; the report layer is the single output format.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; printing is total over any finding set.
// CEP:ASSUMES: none.
// CEP:COST: O(findings log findings) for the sort; O(findings) for printing.
// CEP:EVIDENCE: cargo test -p cep_lint report_sorting.
// CEP:SECURITY: Output contains repository-relative paths only.
//! Findings and report output.

/// One lint finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// 0..3 severity per the 34.2 classes.
    pub severity: i64,
    /// Rule identifier (CEP-LINT-*).
    pub rule: String,
    /// Repository-relative file path.
    pub file: String,
    /// One-based line number; 0 for file-level findings.
    pub line: usize,
    /// Human-readable finding message.
    pub message: String,
}

/// Prints findings in the deterministic 50.1 order.
// CEP:WHAT: Prints findings sorted by file, line, rule, and message.
// CEP:WHY: Deterministic output (6.5) makes CI diffs meaningful; the severity class leads each line for triage.
// CEP:STATUS: complete
// CEP:FAILURE: none; total over any finding set.
// CEP:ASSUMES: none.
// CEP:COST: O(findings log findings).
// CEP:EVIDENCE: cargo test -p cep_lint report_sorting.
pub fn print_findings(findings: &[Finding]) {
    for finding in findings {
        println!(
            "[CEP-LINT] SEVERITY {}: {}:{}: {}: {}",
            finding.severity, finding.file, finding.line, finding.rule, finding.message
        );
    }
}
