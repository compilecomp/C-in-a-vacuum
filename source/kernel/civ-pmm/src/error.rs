// CEP:FILE: source/kernel/civ-pmm/src/error.rs
// CEP:WHAT: Error enum for the physical memory manager covering every failure mode of its public API.
// CEP:WHY: Standards Law 6 requires explicit, bounded failure behavior; one total error enum makes every failure point nameable at call sites and in proofs.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: The type itself represents failures; constructing a value has no failure path. Display never panics and never allocates.
// CEP:ASSUMES: none; the enum is total over its variants.
// CEP:COST: O(1) construction; Display is O(message length), message length bounded per variant, no allocation.
// CEP:EVIDENCE: unit test test_error_display_stable_and_nonempty; Kani harnesses classify the free and construction error paths (proof_free_validates_untrusted_input, proof_new_rejects_bad_config_no_panic).
// CEP:SECURITY: Error variants leak no memory addresses or layout details (design.md CIV-REQ-009 information disclosure); messages are fixed strings.

//! Failure modes of the physical memory manager.

use core::fmt;

/// Every failure the physical memory manager can report.
///
/// No variant indicates a panic path; all are returned values. The set is
/// closed: new failure modes require a new variant plus a registry update
/// (design.md 0.3 completeness rule).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PmmError {
    /// No free frame satisfies the request.
    Exhausted,
    /// Argument structurally invalid: zero length, unaligned base, inverted range, or out-of-bounds index.
    InvalidArgument,
    /// Target frame's allocation bit is zero: never allocated or already freed (double free).
    NotAllocated,
    /// Reserve request overlaps an already-allocated frame; rejected atomically.
    RegionInUse,
    /// Backing bitmap store is smaller than the requested frame count requires.
    BitmapTooSmall,
    /// Checked arithmetic exceeded the u64 domain.
    Overflow,
}

impl fmt::Display for PmmError {
    // CEP:WHAT: Renders each error variant as a fixed, allocation-free description.
    // CEP:WHY: Kernel diagnostics need human-readable failures without heap formatting machinery (standards 25.3.1); fixed strings keep the cost bounded and deterministic.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; Display::fmt is total over all variants and writes into the caller-provided formatter.
    // CEP:ASSUMES: The formatter backend does not panic; guaranteed by core::fmt contract for core formatters.
    // CEP:COST: O(length of one fixed message); longest message 56 bytes (NotAllocated); no allocation, no locks.
    // CEP:EVIDENCE: unit test test_error_display_stable_and_nonempty in source/kernel/civ-pmm/tests/allocator_tests.rs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            PmmError::Exhausted => "physical frame allocator exhausted",
            PmmError::InvalidArgument => "invalid argument to frame allocator",
            PmmError::NotAllocated => "frame is not allocated (double free or unallocated free)",
            PmmError::RegionInUse => "frame region overlaps an existing allocation",
            PmmError::BitmapTooSmall => "backing bitmap smaller than required",
            PmmError::Overflow => "arithmetic overflow in frame computation",
        };
        f.write_str(message)
    }
}

impl core::error::Error for PmmError {}
