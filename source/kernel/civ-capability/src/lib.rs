// CEP:FILE: source/kernel/civ-capability/src/lib.rs
// CEP:WHAT: Crate root for the CIV capability subsystem (design.md 9.1 authority system).
// CEP:WHY: Wires the rights bitset, the capability table, and the error taxonomy behind one no_std surface so the kernel's authority checks depend on a single verified unit with typed failures.
// CEP:CLASS: CEP-1
// CEP:STATUS: partial
// CEP:OWNER: civ-maintainer
// CEP:TICKET: CIV-5
// CEP:FAILURE: none; module wiring only, no runtime failure paths of its own.
// CEP:ASSUMES: One table instance per backing store at a time; enforced by the borrow checker through the &mut [Slot] backing store. Handles are opaque u64 values; holders cannot inspect slot internals beyond the API.
// CEP:COST: compile-time wiring only; no runtime instructions beyond the referenced modules.
// CEP:EVIDENCE: cargo test -p civ-capability; cargo kani -p civ-capability; entries in verification/proof-registry.json.
// CEP:SECURITY: The subsystem is the kernel's authority boundary (design.md 9.1, threat model TM-3); every API treats handles and right requests as untrusted and fails closed.
//! CIV capability subsystem.
//!
//! A fixed-capability table over a caller-provided backing store. Each slot
//! holds the rights, target object, and a generation counter for one
//! capability. A handle packs a slot index and a generation; the generation
//! makes stale or forged handles fail closed at lookup time.
//!
//! # Invariants
//!
//! 1. Attenuation only: every capability created by `derive` holds a rights
//!    set that is a subset of its parent's rights at derivation time.
//! 2. Fail closed: `lookup` accepts exactly the handles whose slot index is
//!    in range, the slot is allocated, and the generation matches; every
//!    other u64 value is rejected with a typed error.
//! 3. Revocation closes access: after `revoke` succeeds, the same handle
//!    never resolves again (the generation has advanced past it).
//! 4. No panic on any input: all index arithmetic is bounds-checked, all
//!    right-set arithmetic is total over u64.
//!
//! # Known subset (CEP:STATUS partial, ticket CIV-5)
//!
//! Cascading revocation (revoking a parent invalidates capabilities derived
//! from it) is not implemented; revocation invalidates only the named
//! handle. Lineage tracking is the declared next increment. All other
//! design.md 9.1 surfaces named below are implemented: representation,
//! allocation, validation, derivation, attenuation, revocation.
//!
//! # Concurrency
//!
//! The table is single-owner and not `Sync`/`Send` beyond the backing
//! store's autotraits; the authority model (design.md 9.1 authority audit)
//! will define its own ordering contract at the process-manager increment.
#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod error;
pub mod rights;
pub mod table;

#[cfg(kani)]
mod proofs;

pub use error::CapabilityError;
pub use rights::{Rights, RIGHT_EXECUTE, RIGHT_GRANT, RIGHT_MAP, RIGHT_READ, RIGHT_WRITE};
pub use table::{CapabilityTable, Handle, SlotEntry};
