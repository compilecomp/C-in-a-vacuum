// CEP:FILE: fixtures/di_compliant.rs
// CEP:WHAT: Self-test fixture for a configured rule scenario.
// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; fixture only.
// CEP:ASSUMES: fixture only.
// CEP:COST: offline; runtime cost irrelevant.
// CEP:EVIDENCE: cep_lint --self-test scenario.
// CEP:SECURITY: fixture contains no secrets.

// CEP:ID: CS-z01
// CEP:WHAT: A long function with an ID.
// CEP:WHY: Exercises the doc-id check in its passing form.
// CEP:STATUS: complete
// CEP:FAILURE: none.
// CEP:ASSUMES: none.
// CEP:COST: offline.
// CEP:EVIDENCE: self-test.
fn long_function() {
    let a = 0;
    let b = 0;
    let c = 0;
    let d = 0;
    let e = 0;
    let f = 0;
    let g = 0;
    let h = 0;
    let i = 0;
    let j = 0;
    let k = 0;
}
