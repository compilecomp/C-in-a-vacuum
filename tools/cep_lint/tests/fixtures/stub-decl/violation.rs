// CEP:FILE: fixtures/sd_violation.rs
// CEP:WHAT: Self-test fixture for a configured rule scenario.
// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; fixture only.
// CEP:ASSUMES: fixture only.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cep_lint --self-test scenario.
// CEP:SECURITY: fixture contains no secrets.

// CEP:WHAT: A stub without owner or ticket.
// CEP:WHY: Exercises the stub-declaration check.
// CEP:STATUS: stub
// CEP:FAILURE: none.
// CEP:ASSUMES: none.
// CEP:COST: offline.
// CEP:EVIDENCE: self-test.
fn stub_function() {
    let value = 0;
}
