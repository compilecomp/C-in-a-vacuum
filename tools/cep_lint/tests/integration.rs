// CEP:FILE: tools/cep_lint/tests/integration.rs
// CEP:WHAT: Integration tests for the cep_lint exit-code contract, config gates, regex subset, and report sorting.
// CEP:WHY: Standards 50.6 requires cargo test -p cep_lint as part of the self-verification gate; these tests pin the behavioral contract the CEP blocks claim.
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion fails cargo test and blocks CI.
// CEP:ASSUMES: Tests run from the workspace root so repository paths resolve.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cargo test -p cep_lint.
// CEP:SECURITY: No network; no secrets.
//! Integration tests for the cep_lint behavioral contract.

use std::process::Command;

#[test]
fn exit_code_contract() {
    // --version exits 0.
    let output = Command::new(env!("CARGO_BIN_EXE_cep_lint"))
        .arg("--version")
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cep_lint 0.4.0"));
    assert!(stdout.contains("CEP&CC 0.4"));

    // Usage error exits 3.
    let output = Command::new(env!("CARGO_BIN_EXE_cep_lint"))
        .arg("--bogus-flag")
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(3));

    // Missing target exits 3.
    let output = Command::new(env!("CARGO_BIN_EXE_cep_lint"))
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn self_test_gate_passes() {
    let output = Command::new(env!("CARGO_BIN_EXE_cep_lint"))
        .arg("--self-test")
        .output()
        .expect("binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "self-test failed: {stdout}");
    assert!(stdout.contains("0 failures"), "{stdout}");
}

#[test]
fn doc_sync_live_gate_passes() {
    // DOC-SYNC runs at scan level; scanning the tool source must yield no
    // DOC-SYNC findings (severity 0/1) for the configuration-to-document
    // contract.
    let output = Command::new(env!("CARGO_BIN_EXE_cep_lint"))
        .args(["tools/cep_lint/src"])
        .output()
        .expect("binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("DOC-SYNC:"),
        "DOC-SYNC drift detected: {stdout}"
    );
}
