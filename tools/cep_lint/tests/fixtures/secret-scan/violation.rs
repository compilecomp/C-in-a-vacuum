// CEP:FILE: fixtures/ss_violation.rs
// CEP:WHAT: Self-test fixture for a configured rule scenario.
// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; fixture only.
// CEP:ASSUMES: fixture only.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cep_lint --self-test scenario.
// CEP:SECURITY: fixture contains no secrets.

fn config() {
    let api_key = "sk-live-1234567890";
}
