// CEP:FILE: source/kernel/civ-pmm/src/frame.rs
// CEP:WHAT: Physical frame identity types: raw physical address, frame number, and half-open frame range.
// CEP:WHY: Index-based frame identity keeps allocator arithmetic in frame units and confines address multiplication to checked boundaries, per design.md 4.1 physical page allocator; a range type prevents start/end pairs from drifting apart at call sites.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: Constructors return PmmError::InvalidArgument or PmmError::Overflow on bad input; no panic on any input; no allocation; no I/O.
// CEP:ASSUMES: FRAME_SIZE_BYTES is the target page size (x86-64 4 KiB baseline, design.md 1.3 Supported page sizes); declared here as the named target configuration constant required by standards Law 7 and to be re-validated against the CPU feature registry at target bring-up (design.md 1.1).
// CEP:COST: O(1) checked arithmetic per construction; no allocation, no locks.
// CEP:EVIDENCE: unit tests frame_* in source/kernel/civ-pmm/tests/allocator_tests.rs; Kani proof proof_new_rejects_bad_config_no_panic.
// CEP:SECURITY: Frame values cross kernel and device boundaries (design.md 4.5); validation is retained in constructors rather than trusted to callers (threat model TM-2).

//! Frame identity for the physical memory manager.

use crate::error::PmmError;

/// Byte size of one allocatable physical frame.
///
/// Named target configuration constant (standards Law 7): x86-64 baseline
/// 4 KiB paging per design.md 1.3 (Supported page sizes). The constant must be re-validated
/// against the discovered CPU paging capabilities at target bring-up;
/// it is not a silent assumption.
// CEP:WHAT: Target frame size in bytes.
// CEP:WHY: A named configuration constant instead of scattered 4096 literals; standards Law 7 bans hard-coded environment assumptions.
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: x86-64 4 KiB paging baseline; enforcement is the target manifest review at bring-up (design.md 1.1 CPU platform), with the page-size authority being design.md 1.3 Supported page sizes.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: design.md 1.3 Supported page sizes; standards Law 7.
pub const FRAME_SIZE_BYTES: u64 = 4096;

/// A raw physical memory address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Wraps a raw physical address without validation; every use site bounds-checks.
    // CEP:WHAT: Wraps a raw physical address into the address newtype.
    // CEP:WHY: A distinct address type prevents accidental mixing with frame numbers and offsets at type level.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none; the value itself carries no precondition, consumers validate.
    // CEP:COST: O(1); single register move, no allocation.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_address_roundtrip.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: the observable behavior is exactly one wrap; any implementation performs the same single move. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw address value.
    // CEP:WHAT: Unwraps the address newtype to the raw u64 value.
    // CEP:WHY: Callers at hardware boundaries need the raw value; keeping the conversion explicit marks those boundaries.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); single register move.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_address_roundtrip.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: identity projection; no cheaper implementation exists. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// True when the address is a multiple of the frame size.
    // CEP:WHAT: Reports whether the address is frame-aligned.
    // CEP:WHY: Allocator construction must reject unaligned bases instead of silently truncating (standards 66.2 war on silent truncation).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total predicate.
    // CEP:ASSUMES: FRAME_SIZE_BYTES is a power of two on supported targets; documented in the target contract.
    // CEP:COST: O(1); one AND on power-of-two targets, one division otherwise.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_new_rejects_unaligned_base.
    #[inline]
    pub const fn is_frame_aligned(self) -> bool {
        self.0.is_multiple_of(FRAME_SIZE_BYTES)
    }

    /// Address of the first byte of frame `number`; checked multiply.
    // CEP:WHAT: Converts a frame number to its starting physical address with overflow checking.
    // CEP:WHY: Frame-to-address multiplication is the one place frame arithmetic can overflow; confining it to a checked constructor (standards 66.4 forgotten wrapping).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::Overflow when number * FRAME_SIZE_BYTES exceeds u64 range.
    // CEP:ASSUMES: none; input may be any u64 (threat model TM-2).
    // CEP:COST: O(1); one checked multiply plus branch.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_address_overflow_rejected; Kani proof proof_new_rejects_bad_config_no_panic.
    // CEP:SECURITY: Frame values feed DMA mapping (design.md 4.5); an unchecked overflow here would alias device memory.
    #[inline]
    pub fn from_frame_number(number: u64) -> Result<Self, PmmError> {
        match number.checked_mul(FRAME_SIZE_BYTES) {
            Some(addr) => Ok(Self(addr)),
            None => Err(PmmError::Overflow),
        }
    }
}

/// A zero-based physical frame number; the frame spans
/// `[number * FRAME_SIZE_BYTES, (number + 1) * FRAME_SIZE_BYTES)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame(u64);

impl Frame {
    /// Wraps a raw frame number without validation; use site validates range.
    // CEP:WHAT: Wraps a raw frame number into the frame newtype.
    // CEP:WHY: Frame numbers are the allocator's index space; a dedicated type stops raw addresses and indices from being interchanged.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none; consumers validate the range against a concrete allocator.
    // CEP:COST: O(1); single register move.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_free_roundtrip.
    #[inline]
    pub const fn new(number: u64) -> Self {
        Self(number)
    }

    /// Returns the raw frame number.
    // CEP:WHAT: Unwraps the frame newtype to the raw frame number.
    // CEP:WHY: Bitmap indexing and diagnostics need the raw index; explicit conversion marks the trust boundary.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); single register move.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_free_roundtrip.
    #[inline]
    pub const fn number(self) -> u64 {
        self.0
    }

    /// Address of the first byte of this frame; checked multiply.
    // CEP:WHAT: Converts the frame to its starting physical address.
    // CEP:WHY: Hardware consumers (page tables, DMA) need addresses; the conversion stays checked at the boundary.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::Overflow on checked-multiply failure.
    // CEP:ASSUMES: none; any frame number is accepted.
    // CEP:COST: O(1); one checked multiply plus branch.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_address_roundtrip.
    #[inline]
    pub fn start_address(self) -> Result<PhysAddr, PmmError> {
        PhysAddr::from_frame_number(self.0)
    }

    /// Frame containing byte address `addr` (floored).
    // CEP:WHAT: Returns the frame that contains the given byte address.
    // CEP:WHY: Region handoff from firmware memory maps (design.md 2.2 memory-map handoff) arrives as byte ranges; conversion to frames must be total and explicit.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total over u64 (floor division cannot panic).
    // CEP:ASSUMES: FRAME_SIZE_BYTES nonzero; guaranteed by its definition.
    // CEP:COST: O(1); one division on non-power-of-two targets, one shift otherwise.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_address_roundtrip.
    #[inline]
    pub const fn containing_address(addr: u64) -> Self {
        Self(addr / FRAME_SIZE_BYTES)
    }
}

/// Half-open frame range `[start, end)`; `end >= start` enforced at construction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameRange {
    start: Frame,
    end: Frame,
}

impl FrameRange {
    /// Constructs a range; rejects `end` below `start`.
    // CEP:WHAT: Constructs a validated half-open frame range.
    // CEP:WHY: A validated range type makes the empty-versus-inverted distinction explicit instead of letting callers derive it inconsistently (standards 14.4 explicitness).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when end < start; empty ranges are valid and carry length zero.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one comparison.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_range_inverted_rejected.
    #[inline]
    pub fn new(start: Frame, end: Frame) -> Result<Self, PmmError> {
        if end.number() < start.number() {
            return Err(PmmError::InvalidArgument);
        }
        Ok(Self { start, end })
    }

    /// First frame of the range.
    // CEP:WHAT: Returns the inclusive start frame of the range.
    // CEP:WHY: Callers need the range origin for bitmap index translation.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_reserve_range_then_alloc_skips.
    #[inline]
    pub const fn start(&self) -> Frame {
        self.start
    }

    /// One-past-last frame of the range.
    // CEP:WHAT: Returns the exclusive end frame of the range.
    // CEP:WHY: Half-open ranges make subtraction total and nesting checks unambiguous.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_reserve_range_then_alloc_skips.
    #[inline]
    pub const fn end(&self) -> Frame {
        self.end
    }

    /// Number of frames covered; zero for an empty range.
    // CEP:WHAT: Returns the length of the range in frames.
    // CEP:WHY: Accounting (design.md 1.3 Algorithms: free-page accounting) operates on lengths; the subtraction is safe because construction validated ordering.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total; the subtraction cannot underflow because end >= start holds by construction.
    // CEP:ASSUMES: The range was produced by FrameRange::new or a literal with end >= start; enforced by construction.
    // CEP:COST: O(1); one subtraction.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_frame_range_inverted_rejected.
    #[inline]
    pub const fn len(&self) -> u64 {
        self.end.0 - self.start.0
    }

    /// True when the range covers no frames.
    // CEP:WHAT: Reports whether the range is empty.
    // CEP:WHY: Reserve and allocation paths distinguish empty requests from real work without a magic sentinel.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total predicate.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one comparison.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_contiguous_zero_len_rejected.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.end.0 == self.start.0
    }

    /// True when `frame` lies within `[start, end)`.
    // CEP:WHAT: Reports whether a frame is contained in the range.
    // CEP:WHY: Ownership checks (design.md 4.1 page ownership management) reduce to containment before any index arithmetic happens.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total predicate.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); two comparisons.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_free_out_of_range_rejected.
    #[inline]
    pub const fn contains(&self, frame: Frame) -> bool {
        frame.0 >= self.start.0 && frame.0 < self.end.0
    }
}
