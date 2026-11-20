// CEP:FILE: source/kernel/civ-pmm/src/allocator.rs
// CEP:WHAT: The frame allocator state machine over the bitmap: construction validation, single-frame allocation with a rotating scan hint, contiguous first-fit allocation, validated free, and atomic region reservation.
// CEP:WHY: design.md 4.1 requires a physical page allocator with page ownership and free-page accounting; a single-owner state machine over a verified bitmap gives every obligation (no double free, no overlap, exact accounting) one provable surface.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: Every operation returns a typed error (PmmError) instead of panicking; failed operations leave state unchanged; no allocation; no I/O.
// CEP:ASSUMES: Single-owner access (not Sync/Send); frame_count fits the backing store (validated in new); base_frame + frame_count does not overflow (validated in new, then immutable, relied on via debug assertion per Law 3).
// CEP:COST: alloc O(1) amortized via rotating hint plus O(words) worst-case wrap; alloc_contiguous O(count/64) first-fit; free O(1); reserve O(len/64); all allocation-free.
// CEP:EVIDENCE: unit tests in source/kernel/civ-pmm/tests/allocator_tests.rs (33 scenarios, adversarial-dominant); Kani proofs proof_alloc_exhaustion_and_uniqueness, proof_free_validates_untrusted_input, proof_contiguous_bounds_and_disjoint, proof_reserve_conflict_is_atomic, proof_bitmap_run_search_consistent, proof_bitmap_set_run_bounds, proof_invariant_popcount_matches_allocated.
// CEP:SECURITY: All inputs untrusted (threat model TM-1, TM-2); free and reserve validate ownership before mutation; bounds checks are retained as required security cost (standards 23.7).

//! Frame allocator state machine.

use crate::bitmap::Bitmap;
use crate::error::PmmError;
use crate::frame::{Frame, FrameRange, PhysAddr, FRAME_SIZE_BYTES};

/// Bitmap-based physical frame allocator over a caller-provided backing store.
///
/// The allocator stores frame numbers, not addresses; bit `i` of the bitmap
/// tracks frame `base_frame + i`. A rotating scan hint gives amortized O(1)
/// single-frame allocation when freed frames are reused promptly.
#[derive(Debug)]
pub struct FrameAllocator<'a> {
    base_frame: u64,
    frame_count: u64,
    bitmap: Bitmap<'a>,
    allocated: u64,
    scan_hint: u64,
}

impl<'a> FrameAllocator<'a> {
    /// Validates configuration, clears the backing store, and returns the
    /// empty allocator.
    // CEP:ID: MEM-a01
    // CEP:WHAT: Constructs a frame allocator over an aligned base address and a caller-provided bitmap backing store.
    // CEP:WHY: Every structural requirement (alignment, nonzero size, backing capacity, no overflow of the frame window) is checked once here so every later operation can rely on a proven-valid state instead of re-checking (23.6.1 no redundant checks on proven conditions).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidArgument for unaligned base or zero frames, BitmapTooSmall for an undersized backing store, Overflow when base_frame + frame_count exceeds u64; on any error no state exists and the backing store is left cleared-untouched only if validation failed before clear_all (documented: validation precedes clear_all, so failed construction never writes the caller's memory).
    // CEP:ASSUMES: The caller owns the backing store exclusively for the allocator's lifetime; enforced by the mutable borrow.
    // CEP:COST: O(words) for the one-time clear; all other work O(1).
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: an allocator over w words cannot guarantee a deterministic fresh state without writing w words once (the backing contents are unconstrained); the construction performs exactly that single pass plus O(1) validation.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_new_rejects_undersized_bitmap, test_new_rejects_unaligned_base, test_new_rejects_overflow_base; Kani proof proof_new_rejects_bad_config_no_panic.
    // CEP:SECURITY: Base address and frame count come from the firmware memory map (design.md 2.2 memory-map handoff) and are untrusted until validated here; unvalidated values would alias device memory (design.md 4.5).
    pub fn new(base: PhysAddr, frame_count: u64, backing: &'a mut [u64]) -> Result<Self, PmmError> {
        if !base.is_frame_aligned() || frame_count == 0 {
            return Err(PmmError::InvalidArgument);
        }
        let required = Bitmap::required_words(frame_count)?;
        if (backing.len() as u64) < required as u64 {
            return Err(PmmError::BitmapTooSmall);
        }
        let base_frame = base.as_u64() / FRAME_SIZE_BYTES;
        let window_top = base_frame
            .checked_add(frame_count)
            .ok_or(PmmError::Overflow)?;
        // The whole window must be addressable: a frame number that cannot
        // be converted to a byte address would fail later at the DMA or
        // page-table boundary (design.md 4.5), so construction rejects it
        // up front instead of leaving a window that is half-usable.
        if window_top.checked_mul(FRAME_SIZE_BYTES).is_none() {
            return Err(PmmError::Overflow);
        }
        let mut bitmap = Bitmap::new(backing);
        bitmap.clear_all();
        Ok(Self {
            base_frame,
            frame_count,
            bitmap,
            allocated: 0,
            scan_hint: 0,
        })
    }

    /// Allocates one free frame; the rotating hint reuses recent holes first.
    // CEP:ID: MEM-a02
    // CEP:WHAT: Allocates a single physical frame via a rotating first-free scan.
    // CEP:WHY: Single-frame allocation is the most frequently invoked allocator path in the current design (design.md 4.1; per-CPU caches arrive at hardware bring-up); the rotating hint makes freed holes O(1) to reuse instead of re-scanning from zero every time, and the wrap search keeps correctness when the hint region is full.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns Exhausted when no frame is free in either scan segment; the retained checked_add on frame-number reconstruction maps a proven-impossible window overflow to Overflow (23.7 retained security cost); state is unchanged on every error path.
    // CEP:ASSUMES: base_frame + scan index arithmetic stays within the frame window validated at construction (checked_add retained at the reconstruction point); behavioral proof surface is Kani proof proof_alloc_exhaustion_and_uniqueness.
    // CEP:COST: O(1) amortized (hint lands on a reusable hole), O(count/64) worst case when the post-hint region is full; no allocation.
    // CEP:OPTIMAL: not-optimal
    // CEP:OPTPROOF: within the non-wrap segment the implementation loads exactly the words between the hint and the answer (the per-call lower bound for that segment); one bit set plus one accounting update per success.
    // CEP:OPTNOTE: after a failed hint segment the wrap re-scan re-loads words the first scan already loaded, so a known cheaper cyclic-cursor implementation exists; the flat bitmap keeps the proof surface small. Optimization tracked as ticket CEP-11. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_hint_wrap, test_alloc_exhaustion_exact; Kani proof proof_alloc_exhaustion_and_uniqueness.
    pub fn alloc(&mut self) -> Result<Frame, PmmError> {
        let index = match self.bitmap.find_zero(self.scan_hint, self.frame_count)? {
            Some(index) => index,
            None => {
                if self.scan_hint == 0 {
                    return Err(PmmError::Exhausted);
                }
                match self.bitmap.find_zero(0, self.scan_hint)? {
                    Some(index) => index,
                    None => return Err(PmmError::Exhausted),
                }
            }
        };
        self.bitmap.set(index)?;
        self.allocated += 1;
        self.scan_hint = if index + 1 >= self.frame_count {
            0
        } else {
            index + 1
        };
        let number = self
            .base_frame
            .checked_add(index)
            .ok_or(PmmError::Overflow)?;
        Ok(Frame::new(number))
    }

    /// Allocates `run_len` physically contiguous frames, first-fit from frame zero.
    // CEP:WHAT: Allocates a contiguous frame range by full-range first-fit search.
    // CEP:WHY: Contiguous requests are rare, cold, and must respect fragmentation reality (a hint-region-local search can miss runs that straddle the hint boundary, standards 66.3 silent fallback ban); a single deterministic full-range first-fit is the simplest provably-complete strategy and its cost is bounded by one bitmap pass.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidArgument for zero or oversized run length; Exhausted when no run of the requested length exists anywhere in the window; state unchanged on error.
    // CEP:ASSUMES: none beyond construction invariants.
    // CEP:COST: O(count/64) word loads plus O(run/64) verification per candidate; candidates never re-examine excluded bits (bitmap::find_zero_run conflict skipping); no allocation.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: a complete first-fit search must certify that no earlier position hosts the run, which requires scanning every word up to the answer at least once (Omega(count/64) loads); the implementation loads exactly that.
    // CEP:OPTNOTE: a hint-bounded search would be faster on average but incomplete across the hint boundary; completeness was chosen over average speed per design.md 4.2 demand-paging correctness and standards 23.7 (correctness beats micro-optimality).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_contiguous_full_capacity, test_alloc_contiguous_first_fit_layout, test_alloc_contiguous_after_fragmentation; Kani proof proof_contiguous_bounds_and_disjoint.
    pub fn alloc_contiguous(&mut self, run_len: u64) -> Result<FrameRange, PmmError> {
        if run_len == 0 || run_len > self.frame_count {
            return Err(PmmError::InvalidArgument);
        }
        let index = match self.bitmap.find_zero_run(0, run_len, self.frame_count)? {
            Some(index) => index,
            None => return Err(PmmError::Exhausted),
        };
        self.bitmap.set_run(index, run_len)?;
        self.allocated += run_len;
        let start_number = self
            .base_frame
            .checked_add(index)
            .ok_or(PmmError::Overflow)?;
        let end_number = self
            .base_frame
            .checked_add(index)
            .ok_or(PmmError::Overflow)?
            .checked_add(run_len)
            .ok_or(PmmError::Overflow)?;
        let start = Frame::new(start_number);
        let end = Frame::new(end_number);
        FrameRange::new(start, end)
    }

    /// Frees a previously allocated frame; rejects never-allocated and already-freed frames.
    // CEP:WHAT: Returns one frame to the free pool after validating ownership.
    // CEP:WHY: Double-free and out-of-window frees are the classic allocator exploits (threat model TM-1, TM-2); the read-validate-clear sequence makes rejection total and leaves state untouched on failure, discharging the design.md 4.5 no-double-free obligation.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidArgument when the frame number is outside the managed window; NotAllocated when the frame's bit is zero (never allocated or already freed); in both cases the allocator state is bit-for-bit unchanged.
    // CEP:ASSUMES: The frame window top (base_frame + frame_count) cannot overflow; validated at construction and immutable since, enforced here by debug assertion per Law 3.
    // CEP:COST: O(1); two compares, one bounds-checked bit read, one clear, one accounting update.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: correct double-free rejection requires reading the bit before clearing; the implementation performs exactly one read, one clear, and the two range comparisons any correct ownership check needs.
    // CEP:SECURITY: Ownership validation is retained security cost (standards 23.7); removing it would enable cross-owner frame aliasing (design.md 4.5 no use-after-free).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_double_free_rejected, test_free_out_of_range_rejected, test_free_then_realloc_returns_same_frame; Kani proof proof_free_validates_untrusted_input over the full u64 input range.
    pub fn free(&mut self, frame: Frame) -> Result<(), PmmError> {
        debug_assert!(
            self.base_frame.checked_add(self.frame_count).is_some(),
            "allocator frame window overflow; construction invariant violated"
        );
        let number = frame.number();
        if number < self.base_frame || number >= self.base_frame + self.frame_count {
            return Err(PmmError::InvalidArgument);
        }
        let index = number - self.base_frame;
        if !self.bitmap.get(index)? {
            return Err(PmmError::NotAllocated);
        }
        self.bitmap.clear(index)?;
        self.allocated -= 1;
        self.scan_hint = index;
        Ok(())
    }

    /// Marks a frame range as allocated without handing it out (kernel-reserved regions).
    // CEP:ID: MEM-a03
    // CEP:WHAT: Atomically reserves a frame range for structures the kernel must keep out of the free pool.
    // CEP:WHY: Firmware-reserved and kernel-image frames (design.md 1.3 Reserved-memory manager; handoff data from the design.md 2.2 memory-map reader) must never surface from alloc; the conflict check runs before any mutation so a rejected reservation cannot leave a half-marked region (standards 66.7 forgotten cleanup).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidArgument for empty ranges or ranges outside the window; RegionInUse when any frame in the range is already allocated; state is bit-for-bit unchanged on every error path.
    // CEP:ASSUMES: The range end is exclusive and within the window; validated here against the construction-validated window top.
    // CEP:COST: O(len/64) conflict scan plus O(len/64) run set; one pass each; no allocation.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: an atomic reserve must examine every word of the range for conflicts before writing any of them, and must write every word it accepts; the implementation performs exactly one conflict pass and one write pass.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_reserve_range_conflict_atomic, test_reserve_range_then_alloc_skips; Kani proof proof_reserve_conflict_is_atomic.
    pub fn reserve_range(&mut self, range: FrameRange) -> Result<(), PmmError> {
        if range.is_empty() {
            return Err(PmmError::InvalidArgument);
        }
        let start_number = range.start().number();
        let end_number = range.end().number();
        let window_top = self.base_frame + self.frame_count;
        if start_number < self.base_frame || end_number > window_top {
            return Err(PmmError::InvalidArgument);
        }
        let start_index = start_number - self.base_frame;
        let end_index = end_number - self.base_frame;
        if self.bitmap.find_one(start_index, end_index)?.is_some() {
            return Err(PmmError::RegionInUse);
        }
        self.bitmap.set_run(start_index, end_index - start_index)?;
        self.allocated += end_index - start_index;
        self.scan_hint = start_index;
        Ok(())
    }

    /// Total frames managed by this allocator.
    // CEP:WHAT: Returns the managed frame count.
    // CEP:WHY: Accounting consumers (design.md 4.1 memory statistics) need the window size without touching allocator internals.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_stats_consistency.
    #[inline]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Frames currently allocated or reserved.
    // CEP:WHAT: Returns the allocated-frame accounting counter.
    // CEP:WHY: Invariant 2 of the crate ties this counter to the bitmap popcount; exposing it makes the invariant externally checkable.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: The counter is maintained by construction and proof proof_invariant_popcount_matches_allocated.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_popcount_matches_allocated.
    #[inline]
    pub const fn allocated_count(&self) -> u64 {
        self.allocated
    }

    /// Frames currently free (`frame_count - allocated_count`).
    // CEP:WHAT: Returns the free-frame count implied by the accounting counter.
    // CEP:WHY: Allocation policy and diagnostics consume the free count; deriving it avoids a third mutable counter whose consistency would need separate proof.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor; the subtraction cannot underflow because the invariant allocated <= frame_count holds by construction and proof.
    // CEP:ASSUMES: Invariant allocated_count <= frame_count; enforced by construction (every mutation adds a verified-free run within the window) and Kani proof proof_invariant_popcount_matches_allocated.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_popcount_matches_allocated.
    #[inline]
    pub const fn free_count(&self) -> u64 {
        self.frame_count - self.allocated
    }

    /// Frame number of bitmap bit zero (first managed frame).
    // CEP:WHAT: Returns the base frame number of the managed window.
    // CEP:WHY: Callers translating between frames and window-relative indices need the base; exposing it avoids duplicating the alignment arithmetic at call sites.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_boundary_first_and_last_frame.
    #[inline]
    pub const fn base_frame(&self) -> u64 {
        self.base_frame
    }

    /// Popcount of the allocation bitmap; the mechanical witness for crate invariant 2.
    // CEP:WHAT: Counts set bits directly in the bitmap, independent of the accounting counter.
    // CEP:WHY: Invariant telemetry (design.md 4.1 free-page accounting) requires a second, differently-computed source of truth; disagreements between counter and popcount are detectable corruption, not silent drift (standards 66.1 silence is a defect).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(words); one popcount per word.
    // CEP:EVIDENCE: Kani proof proof_invariant_popcount_matches_allocated; cargo test -p civ-pmm test_stats_consistency.
    #[inline]
    pub fn allocated_popcount(&self) -> u64 {
        self.bitmap.popcount()
    }
}
