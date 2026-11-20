// CEP:FILE: fixtures/mn_compliant.rs
// CEP:WHAT: Self-test fixture for a configured rule scenario.
// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; fixture only.
// CEP:ASSUMES: fixture only.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cep_lint --self-test scenario.
// CEP:SECURITY: fixture contains no secrets.

const WIDTH: u64 = 64;
fn compute() {
    let width = WIDTH;
}
