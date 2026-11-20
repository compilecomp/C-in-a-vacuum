// CEP:FILE: fixtures/hb_violation.rs
// CEP:WHAT: Hot fixture with a banned token.
// CEP:WHY: Exercises the banned-token check in its violating form.
// CEP:CLASS: CEP-0
// CEP:STATUS: complete
// CEP:FAILURE: none.
// CEP:ASSUMES: none.
// CEP:COST: offline.
// CEP:EVIDENCE: self-test.
// CEP:SECURITY: none.
fn hot_function() {
    let text = String::new();
}
