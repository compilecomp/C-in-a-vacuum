// CEP:FILE: source/kernel/civ-capability/src/error.rs
// CEP:WHAT: Error enum for the capability subsystem covering every failure mode of its public API.
// CEP:WHY: Standards Law 6 requires explicit, bounded failure behavior; one total error enum makes every authority-check failure nameable at call sites and in proofs.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: The type itself represents failures; constructing a value has no failure path. Display never panics and never allocates.
// CEP:ASSUMES: none; the enum is total over its variants.
// CEP:COST: O(1) construction; Display is O(message length), longest message 45 bytes (StaleHandle); no allocation.
// CEP:EVIDENCE: unit test test_capability_error_display in source/kernel/civ-capability/tests/capability_tests.rs.
// CEP:SECURITY: Error variants leak no slot contents, generations, or object identifiers beyond the failing handle itself (design.md CIV-REQ-009); messages are fixed strings.

//! Failure modes of the capability subsystem.

use core::fmt;

/// Every failure the capability subsystem can report.
///
/// No variant indicates a panic path; all are returned values. The set is
/// closed: new failure modes require a new variant plus a registry update
/// (design.md 0.3 completeness rule).
// CEP:WHAT: The closed error taxonomy of the capability subsystem.
// CEP:WHY: One total error enum makes every authority-check failure nameable at call sites and in proofs, satisfying the bounded-failure law.
// CEP:STATUS: complete
// CEP:FAILURE: none; the type represents failures and has no failure path of its own.
// CEP:ASSUMES: none; the variant set is total over the declared failure modes.
// CEP:COST: O(1) construction; Display is O(longest message 45 bytes); no allocation.
// CEP:EVIDENCE: unit test test_capability_error_display in tests/capability_tests.rs.
// CEP:SECURITY: Variants leak no slot contents, generations, or object identifiers beyond the failing handle (design.md CIV-REQ-009).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CapabilityError {
    /// Argument structurally invalid: zero-length backing store.
    InvalidArgument,
    /// The handle's slot index is outside the table.
    InvalidSlot,
    /// The handle's slot is not currently allocated.
    SlotEmpty,
    /// The handle's generation does not match the slot's current generation: the handle is stale or forged.
    StaleHandle,
    /// The table has no free slot.
    TableFull,
    /// A derive request asked for rights the parent does not hold.
    Escalation,
    /// The parent handle needed by an operation did not resolve.
    ParentInvalid,
    /// A derive request asked for rights the parent is not allowed to delegate (no RIGHT_GRANT).
    GrantDenied,
}

impl fmt::Display for CapabilityError {
    // CEP:WHAT: Renders each error variant as a fixed, allocation-free description.
    // CEP:WHY: Kernel diagnostics need human-readable authority failures without heap formatting (standards 25.3.1); fixed strings keep cost bounded and deterministic.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; Display::fmt is total over all variants and writes into the caller-provided formatter.
    // CEP:ASSUMES: The formatter backend does not panic; guaranteed by core::fmt contract for core formatters.
    // CEP:COST: O(length of one fixed message); longest message 45 bytes (StaleHandle); no allocation, no locks.
    // CEP:EVIDENCE: unit test test_capability_error_display in source/kernel/civ-capability/tests/capability_tests.rs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            CapabilityError::InvalidArgument => "invalid argument to capability table",
            CapabilityError::InvalidSlot => "capability slot index outside table",
            CapabilityError::SlotEmpty => "capability slot is not allocated",
            CapabilityError::StaleHandle => "stale capability handle (generation mismatch)",
            CapabilityError::TableFull => "capability table has no free slot",
            CapabilityError::Escalation => "derive request exceeds parent rights",
            CapabilityError::ParentInvalid => "parent capability handle did not resolve",
            CapabilityError::GrantDenied => "parent lacks the grant right for derivation",
        };
        f.write_str(message)
    }
}

impl core::error::Error for CapabilityError {}
