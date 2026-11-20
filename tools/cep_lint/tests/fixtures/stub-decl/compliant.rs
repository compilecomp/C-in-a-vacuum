// CEP:FILE: fixtures/sd_compliant.rs
// CEP:WHAT: Self-test fixture for a configured rule scenario.
// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; fixture only.
// CEP:ASSUMES: fixture only.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cep_lint --self-test scenario.
// CEP:SECURITY: fixture contains no secrets.

// CEP:WHAT: A stub with owner and ticket.
// CEP:WHY: Exercises the stub-declaration check in its passing form.
// CEP:STATUS: stub
// CEP:OWNER: maintainer
// CEP:TICKET: CEP-90
// CEP:FAILURE: none.
// CEP:ASSUMES: none.
// CEP:COST: offline.
// CEP:EVIDENCE: self-test.
fn stub_function() {
    let value = 0;
}
