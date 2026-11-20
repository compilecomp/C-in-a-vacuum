// CEP:FILE: tools/cep_lint/src/lib.rs
// CEP:WHAT: Library surface for the cep_lint mechanism cores used by unit tests.
// CEP:WHY: The 50.6 gate requires cargo test -p cep_lint; exposing config and regexlite as a library makes the mechanism cores unit-testable while the binary stays the CLI.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; module re-exports only.
// CEP:ASSUMES: none.
// CEP:COST: compile-time wiring only.
// CEP:EVIDENCE: cargo test -p cep_lint.
// CEP:SECURITY: none.
//! cep_lint mechanism library.

pub mod config;
pub mod function;
pub mod regexlite;
pub mod report;
pub mod scan;
