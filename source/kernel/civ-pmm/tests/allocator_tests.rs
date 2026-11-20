// CEP:FILE: source/kernel/civ-pmm/tests/allocator_tests.rs
// CEP:WHAT: Adversarial-dominant unit and property test suite for the physical memory manager.
// CEP:WHY: Standards 61.1 (the happy path is a lie) and 61.4 (reality curve) require the suite to be dominated by worst-case and adversarial scenarios; design.md 19.1 requires unit tests for every subsystem.
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion fails the cargo test run and blocks CI; no test mutates shared state.
// CEP:ASSUMES: Tests run on the host target; FRAME_SIZE_BYTES is 4096 there. Test-only code may use std allocation (CEP-1/CC classes).
// CEP:COST: offline; runtime cost irrelevant (CEP-2-class evidence generation).
// CEP:EVIDENCE: cargo test -p civ-pmm (recorded in verification/evidence/).
// CEP:SECURITY: Scenarios model threat-model TM-1/TM-2 adversaries: untrusted indices, double frees, exhaustion, and fragmented windows.
// CEP:TEST-CLASS: Unit + property (adversarial-dominant)
// CEP:TEST-TARGET: civ-pmm frame allocator, bitmap, frame identity
// CEP:TEST-QUOTA: 33 scenarios per change; Kani bounded proofs per proofs.rs (8 harnesses); targeted fuzz harness deferred to the cep_hunt infrastructure (ticket CEP-90), with Kani bounded proofs as the declared substitute within their bounds
// CEP:TEST-DISTRIBUTION: 24 worst-case/adversarial (73%), 8 middle (24%), 1 happy (3%)
// CEP:TEST-SCALING: Every new failure mode adds its regression matrix (standards 61.3, 50 tests per confirmed bug)
// CEP:TEST-EVIDENCE: cargo test -p civ-pmm output recorded in verification/evidence/test-run.log

//! Adversarial-dominant tests for the physical memory manager.

use civ_pmm::{Frame, FrameAllocator, FrameRange, PhysAddr, PmmError, FRAME_SIZE_BYTES};

/// Aligned proof base: 1 GiB.
const BASE: u64 = 0x4000_0000;
/// Frame number of BASE.
const BASE_FRAME: u64 = BASE / FRAME_SIZE_BYTES;

/// Builds an allocator over `words` bitmap words. The backing store is
/// leaked deliberately: it stands in for kernel-owned bitmap memory, and
/// the allocator API is allocation-free regardless of test setup.
fn build(count: u64, words: usize) -> FrameAllocator<'static> {
    let backing: &'static mut [u64] = Box::leak(vec![0u64; words].into_boxed_slice());
    FrameAllocator::new(PhysAddr::new(BASE), count, backing)
        .expect("test configuration must be structurally valid")
}

// ---------------------------------------------------------------------------
// Construction validation (worst-case/adversarial)
// ---------------------------------------------------------------------------

// CEP:WHAT: Rejects construction over an unaligned base address.
// CEP:WHY: Guards TM-1/TM-2: firmware handoff addresses are untrusted; silent truncation would alias device memory (standards 66.2).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_new_rejects_unaligned_base() {
    let mut backing = [0u64; 1];
    let result = FrameAllocator::new(PhysAddr::new(BASE + 1), 64, &mut backing);
    assert_eq!(result.unwrap_err(), PmmError::InvalidArgument);
}

// CEP:WHAT: Rejects construction with a zero-frame window.
// CEP:WHY: A zero-window allocator would make every operation degenerate; explicit rejection keeps the frame_count > 0 invariant dischargeable.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_new_rejects_zero_frames() {
    let mut backing = [0u64; 1];
    let result = FrameAllocator::new(PhysAddr::new(BASE), 0, &mut backing);
    assert_eq!(result.unwrap_err(), PmmError::InvalidArgument);
}

// CEP:WHAT: Rejects construction when the backing store is smaller than the frame count requires.
// CEP:WHY: Guards the BitmapTooSmall classification boundary at the exact 64-bit word fit.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_new_rejects_undersized_bitmap() {
    // 65 frames need 2 words; one word must be rejected.
    let mut backing = [0u64; 1];
    let result = FrameAllocator::new(PhysAddr::new(BASE), 65, &mut backing);
    assert_eq!(result.unwrap_err(), PmmError::BitmapTooSmall);
}

// CEP:WHAT: Rejects construction when the window top address is not representable in u64.
// CEP:WHY: A half-addressable window would fail only at the DMA or page-table boundary; construction rejects it up front (design.md 4.5).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_new_rejects_overflow_base() {
    // Aligned address within 4096 of u64::MAX: base_frame + count overflows.
    let top_aligned: u64 = u64::MAX - (u64::MAX % FRAME_SIZE_BYTES);
    let mut backing = [0u64; 1];
    let result = FrameAllocator::new(PhysAddr::new(top_aligned), 2, &mut backing);
    assert_eq!(result.unwrap_err(), PmmError::Overflow);
}

// CEP:WHAT: Accepts construction at the exact one-word fit for 64 frames.
// CEP:WHY: Boundary counterpart of the undersized rejection; proves the capacity check is exact, not padded.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_new_accepts_exact_bitmap_fit() {
    // Boundary: 64 frames need exactly one word.
    let allocator = build(64, 1);
    assert_eq!(allocator.frame_count(), 64);
    assert_eq!(allocator.free_count(), 64);
}

// ---------------------------------------------------------------------------
// Exhaustion and uniqueness (worst-case)
// ---------------------------------------------------------------------------

// CEP:WHAT: Allocates every frame, then verifies exhaustion is reported at exactly the boundary.
// CEP:WHY: Guards the Exhausted classification and exact accounting at the full window.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_exhaustion_exact() {
    let mut allocator = build(8, 1);
    for step in 0..8 {
        assert!(allocator.alloc().is_ok(), "step {step} must allocate");
    }
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
    assert_eq!(allocator.allocated_count(), 8);
}

// CEP:WHAT: Allocates 200 frames and verifies all returned frame numbers are distinct.
// CEP:WHY: Guards the no-double-allocation obligation (design.md 4.5) across word boundaries.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_returns_disjoint_frames() {
    let mut allocator = build(200, 4);
    let mut numbers: Vec<u64> = Vec::new();
    for _ in 0..200 {
        numbers.push(allocator.alloc().unwrap().number());
    }
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers.len(), 200, "frames must be distinct");
}

// CEP:ID: MEM-a15
// CEP:WHAT: Exercises the first frame, the window top frame, and the exact exhaustion boundary of a non-multiple-of-64 window.
// CEP:WHY: Off-by-one cliffs concentrate at window boundaries; 129 frames span three words so the last word is partially used.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_boundary_first_and_last_frame() {
    let count = 129u64; // spans three words, non-multiple of 64
    let mut allocator = build(count, 3);
    let first = allocator.alloc().unwrap();
    assert_eq!(first.number(), BASE_FRAME);
    for _ in 1..count {
        allocator.alloc().unwrap();
    }
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
    // The last allocated frame is the window top minus one; verify by
    // freeing it and observing the window-local index round-trip.
    let top = Frame::new(BASE_FRAME + count - 1);
    allocator.free(top).unwrap();
    let realloc = allocator.alloc().unwrap();
    assert_eq!(realloc.number(), top.number());
}

// ---------------------------------------------------------------------------
// Free validation (worst-case/adversarial)
// ---------------------------------------------------------------------------

// CEP:WHAT: Verifies a double free is rejected with state unchanged and the frame reusable exactly once.
// CEP:WHY: Guards the design.md 4.5 no-double-free obligation and threat model TM-2.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_double_free_rejected() {
    let mut allocator = build(16, 1);
    let frame = allocator.alloc().unwrap();
    allocator.free(frame).unwrap();
    assert!(matches!(allocator.free(frame), Err(PmmError::NotAllocated)));
    assert_eq!(
        allocator.allocated_count(),
        0,
        "rejected free must not mutate"
    );
    // The frame is reusable exactly once more after the rejected double free.
    let realloc = allocator.alloc().unwrap();
    assert_eq!(realloc.number(), frame.number());
}

// CEP:WHAT: Verifies freeing a never-allocated in-window frame is rejected.
// CEP:WHY: Distinguishes the NotAllocated classification for never-allocated frames from the out-of-window case.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_free_never_allocated_rejected() {
    let mut allocator = build(16, 1);
    let never = Frame::new(BASE_FRAME + 7);
    assert!(matches!(allocator.free(never), Err(PmmError::NotAllocated)));
    assert_eq!(allocator.allocated_count(), 0);
}

// CEP:WHAT: Verifies frees below and above the window are rejected with InvalidArgument and no mutation.
// CEP:WHY: Guards TM-1: out-of-window indices must never reach bitmap indexing.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_free_out_of_range_rejected() {
    let mut allocator = build(16, 1);
    allocator.alloc().unwrap();
    let below = Frame::new(BASE_FRAME - 1);
    let above = Frame::new(BASE_FRAME + 16);
    assert!(matches!(
        allocator.free(below),
        Err(PmmError::InvalidArgument)
    ));
    assert!(matches!(
        allocator.free(above),
        Err(PmmError::InvalidArgument)
    ));
    assert_eq!(
        allocator.allocated_count(),
        1,
        "rejected frees must not mutate"
    );
}

// CEP:WHAT: Verifies freeing a frame and reallocating returns exactly the same frame.
// CEP:WHY: Guards the hint-reuse contract that makes alloc O(1) amortized.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_free_then_realloc_returns_same_frame() {
    let mut allocator = build(16, 1);
    let frame = allocator.alloc().unwrap();
    allocator.free(frame).unwrap();
    let realloc = allocator.alloc().unwrap();
    assert_eq!(realloc.number(), frame.number());
}

// CEP:WHAT: Verifies the rotating hint reuses the newest hole first and wraps to recover older holes before reporting exhaustion.
// CEP:WHY: Guards wrap-search correctness of alloc against the classic hint-skips-holes defect.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_hint_wrap() {
    // Fill, free an interior frame, then allocate: the rotating hint must
    // wrap and find the interior hole even though it precedes the hint.
    let mut allocator = build(8, 1);
    let mut frames = Vec::new();
    for _ in 0..8 {
        frames.push(allocator.alloc().unwrap());
    }
    allocator.free(frames[3]).unwrap();
    allocator.free(frames[6]).unwrap();
    // The hint rests on the most recent hole (frame 6); the wrap search
    // then recovers the older hole (frame 3) before reporting exhaustion.
    assert_eq!(allocator.alloc().unwrap().number(), frames[6].number());
    assert_eq!(allocator.alloc().unwrap().number(), frames[3].number());
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
}

// ---------------------------------------------------------------------------
// Contiguous allocation (worst-case fragmentation)
// ---------------------------------------------------------------------------

// CEP:WHAT: Rejects contiguous allocation of length zero.
// CEP:WHY: Zero-length runs are structurally invalid in find_zero_run; the allocator surfaces InvalidArgument before any scan.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_zero_len_rejected() {
    let mut allocator = build(8, 1);
    assert!(matches!(
        allocator.alloc_contiguous(0),
        Err(PmmError::InvalidArgument)
    ));
}

// CEP:WHAT: Rejects contiguous allocation longer than the window.
// CEP:WHY: An oversized request can never fit; rejection is structural, not exhaustion.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_oversized_rejected() {
    let mut allocator = build(8, 1);
    assert!(matches!(
        allocator.alloc_contiguous(9),
        Err(PmmError::InvalidArgument)
    ));
}

// CEP:WHAT: Allocates the full window contiguously and verifies exhaustion afterwards.
// CEP:WHY: Boundary case run_len == frame_count in one pass over one word.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_full_capacity() {
    let mut allocator = build(64, 1);
    let range = allocator.alloc_contiguous(64).unwrap();
    assert_eq!(range.len(), 64);
    assert_eq!(range.start().number(), BASE_FRAME);
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
}

// CEP:WHAT: Verifies first-fit placement inside a manually created hole.
// CEP:WHY: Guards deterministic lowest-position placement, the property proofs rely on for disjointness.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_first_fit_layout() {
    // Allocate 0..10, free 2..5 (a hole of 3), request 3: first-fit must
    // return the lowest position that hosts the run.
    let mut allocator = build(16, 1);
    let mut frames = Vec::new();
    for _ in 0..10 {
        frames.push(allocator.alloc().unwrap());
    }
    for frame in &frames[2..5] {
        allocator.free(*frame).unwrap();
    }
    let range = allocator.alloc_contiguous(3).unwrap();
    assert_eq!(range.start().number(), frames[2].number());
    assert_eq!(range.len(), 3);
}

// CEP:WHAT: Verifies a two-frame request fails under alternating single-frame holes while single frames still succeed.
// CEP:WHY: Worst-case fragmentation pattern; guards against runs spanning allocated frames (the bitmap mask regression).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_after_fragmentation() {
    // Worst case: alternating single-frame holes. No two adjacent frames
    // are free, so any run of 2 must fail while runs of 1 still succeed.
    let mut allocator = build(32, 1);
    let mut frames = Vec::new();
    for _ in 0..32 {
        frames.push(allocator.alloc().unwrap());
    }
    // Free every even-indexed frame: holes never touch.
    for index in (0..32).step_by(2) {
        allocator.free(frames[index]).unwrap();
    }
    assert!(matches!(
        allocator.alloc_contiguous(2),
        Err(PmmError::Exhausted)
    ));
    let single = allocator.alloc_contiguous(1).unwrap();
    assert!(single.len() == 1);
}

// CEP:WHAT: Verifies contiguous allocation reports exhaustion on a full window.
// CEP:WHY: Complements the single-frame exhaustion path for the contiguous code path.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_contiguous_rejects_after_exhaustion() {
    let mut allocator = build(4, 1);
    for _ in 0..4 {
        allocator.alloc().unwrap();
    }
    assert!(matches!(
        allocator.alloc_contiguous(1),
        Err(PmmError::Exhausted)
    ));
}

// ---------------------------------------------------------------------------
// Reservation (atomicity under conflict)
// ---------------------------------------------------------------------------

// CEP:WHAT: Verifies a reservation overlapping an allocation is rejected atomically with popcount and counter unchanged.
// CEP:WHY: Guards reserve atomicity: no half-marked regions after a rejected reserve (standards 66.7).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_reserve_range_conflict_atomic() {
    let mut allocator = build(16, 1);
    let mut frames = Vec::new();
    for _ in 0..16 {
        frames.push(allocator.alloc().unwrap());
    }
    // Range overlaps allocated frame 8: rejected, state unchanged.
    let range = FrameRange::new(frames[6], frames[11]).unwrap();
    assert!(matches!(
        allocator.reserve_range(range),
        Err(PmmError::RegionInUse)
    ));
    assert_eq!(allocator.allocated_count(), 16);
    assert_eq!(allocator.allocated_popcount(), 16);
}

// CEP:WHAT: Rejects reservation of an empty range.
// CEP:WHY: Empty ranges are structurally invalid; the empty check precedes any scan.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_reserve_range_empty_rejected() {
    let mut allocator = build(8, 1);
    let frame = Frame::new(BASE_FRAME + 3);
    let empty = FrameRange::new(frame, frame).unwrap();
    assert!(matches!(
        allocator.reserve_range(empty),
        Err(PmmError::InvalidArgument)
    ));
}

// CEP:WHAT: Rejects reservation of a range extending past the window top.
// CEP:WHY: Guards the window containment check for ranges, complementing per-frame checks.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_reserve_range_out_of_window_rejected() {
    let mut allocator = build(8, 1);
    let range = FrameRange::new(Frame::new(BASE_FRAME + 6), Frame::new(BASE_FRAME + 10)).unwrap();
    assert!(matches!(
        allocator.reserve_range(range),
        Err(PmmError::InvalidArgument)
    ));
}

// CEP:WHAT: Verifies reserved frames are never handed out by alloc and the rest of the window remains usable.
// CEP:WHY: Guards the reserved-memory contract (design.md 1.3 Reserved-memory manager).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_reserve_range_then_alloc_skips() {
    let mut allocator = build(16, 1);
    let range = FrameRange::new(Frame::new(BASE_FRAME + 4), Frame::new(BASE_FRAME + 8)).unwrap();
    allocator.reserve_range(range).unwrap();
    assert_eq!(allocator.allocated_count(), 4);
    // Reserved frames are never handed out.
    for _ in 0..12 {
        let frame = allocator.alloc().unwrap().number();
        assert!(!(BASE_FRAME + 4..BASE_FRAME + 8).contains(&frame));
    }
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
}

// ---------------------------------------------------------------------------
// Accounting invariants (middle class: consistency under load)
// ---------------------------------------------------------------------------

// CEP:WHAT: Verifies counter, popcount, and the window-sum invariant agree after every allocation step and at exhaustion.
// CEP:WHY: Crate invariant 2 (allocated == popcount) observed across a 100-frame workload.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_stats_consistency() {
    let mut allocator = build(100, 2);
    for step in 0..100 {
        allocator.alloc().unwrap();
        assert_eq!(allocator.allocated_count(), step + 1);
        assert_eq!(allocator.allocated_popcount(), step + 1);
        assert_eq!(
            allocator.allocated_count() + allocator.free_count(),
            allocator.frame_count()
        );
    }
    for _ in 0..50 {
        let frame = allocator.alloc().unwrap_err();
        assert!(matches!(frame, PmmError::Exhausted));
    }
}

// CEP:WHAT: Verifies bit 64 of a one-word bitmap is unreachable through any allocator path.
// CEP:WHY: Guards the capacity boundary between bitmap capacity and managed frames.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_bitmap_out_of_bounds_rejected() {
    // 64 frames over one word: bit 64 is out of bounds and must not be
    // settable through any public path.
    let mut allocator = build(64, 1);
    for _ in 0..64 {
        allocator.alloc().unwrap();
    }
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
    let outside = Frame::new(BASE_FRAME + 64);
    assert!(matches!(
        allocator.free(outside),
        Err(PmmError::InvalidArgument)
    ));
}

// ---------------------------------------------------------------------------
// Frame identity (middle class: conversions)
// ---------------------------------------------------------------------------

// CEP:WHAT: Verifies frame-to-address and address-to-frame conversions round-trip at a known boundary.
// CEP:WHY: Guards the frame identity arithmetic used at all hardware boundaries.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_frame_address_roundtrip() {
    let frame = Frame::containing_address(BASE + FRAME_SIZE_BYTES + 128);
    assert_eq!(frame.number(), BASE_FRAME + 1);
    let address = frame.start_address().unwrap();
    assert_eq!(address.as_u64(), BASE + FRAME_SIZE_BYTES);
    assert!(address.is_frame_aligned());
}

// CEP:WHAT: Verifies frame-to-address conversion reports Overflow at the u64 edge.
// CEP:WHY: Guards the checked multiply at the conversion boundary (standards 66.4).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_frame_address_overflow_rejected() {
    let last = Frame::new(u64::MAX);
    assert_eq!(last.start_address().unwrap_err(), PmmError::Overflow);
    assert_eq!(
        PhysAddr::from_frame_number(u64::MAX).unwrap_err(),
        PmmError::Overflow
    );
}

// CEP:WHAT: Verifies an inverted range is rejected and alignment predicates classify correctly.
// CEP:WHY: Guards range construction validation and the alignment predicate.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_frame_range_inverted_rejected() {
    let high = Frame::new(10);
    let low = Frame::new(3);
    assert_eq!(
        FrameRange::new(high, low).unwrap_err(),
        PmmError::InvalidArgument
    );
    assert!(!PhysAddr::new(BASE + 7).is_frame_aligned());
    assert!(PhysAddr::new(BASE).is_frame_aligned());
}

// ---------------------------------------------------------------------------
// Direct bitmap bounds and range containment (adversarial: OOB branches)
// ---------------------------------------------------------------------------

// CEP:ID: MEM-a17
// CEP:WHAT: Every bitmap primitive rejects out-of-capacity indices and zero-length runs directly, with no mutation after failures.
// CEP:WHY: The allocator clamps indices to the managed window, so allocator tests cannot reach the bitmap capacity branches; driving the bitmap API directly covers the InvalidArgument branches documented in each FAILURE field.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; one-word bitmap (capacity 64).
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/test-run.log.
#[test]
fn test_bitmap_direct_out_of_bounds_rejected() {
    // Direct coverage for the out-of-capacity branches of get/set/clear,
    // which the allocator's window clamping never reaches.
    let mut backing = [0u64; 1];
    let mut bitmap = civ_pmm::Bitmap::new(&mut backing);
    assert_eq!(bitmap.bits_capacity(), 64);
    assert!(matches!(bitmap.get(64), Err(PmmError::InvalidArgument)));
    assert!(matches!(bitmap.set(64), Err(PmmError::InvalidArgument)));
    assert!(matches!(bitmap.clear(64), Err(PmmError::InvalidArgument)));
    assert!(matches!(
        bitmap.find_zero(0, 65),
        Err(PmmError::InvalidArgument)
    ));
    assert!(matches!(
        bitmap.find_one(0, 65),
        Err(PmmError::InvalidArgument)
    ));
    assert!(matches!(
        bitmap.find_zero_run(0, 1, 65),
        Err(PmmError::InvalidArgument)
    ));
    assert!(matches!(
        bitmap.set_run(60, 5),
        Err(PmmError::InvalidArgument)
    ));
    assert!(matches!(
        bitmap.set_run(0, 0),
        Err(PmmError::InvalidArgument)
    ));
    // Failed operations must not mutate the bitmap.
    assert_eq!(bitmap.popcount(), 0);
}

// CEP:ID: MEM-a18
// CEP:WHAT: FrameRange::contains classification at both boundaries and for an empty range.
// CEP:WHY: contains is the ownership predicate future page-table consumers rely on (design.md 4.2); pinning it while it awaits its first kernel caller prevents silent drift (standards 34.4 coverage-or-delete).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/test-run.log.
#[test]
fn test_frame_range_contains() {
    // Direct coverage for FrameRange::contains, the ownership predicate
    // used by future page-table consumers (design.md 4.2).
    let range = FrameRange::new(Frame::new(10), Frame::new(14)).unwrap();
    assert!(range.contains(Frame::new(10)));
    assert!(range.contains(Frame::new(13)));
    assert!(!range.contains(Frame::new(14)));
    assert!(!range.contains(Frame::new(9)));
    let empty = FrameRange::new(Frame::new(5), Frame::new(5)).unwrap();
    assert!(!empty.contains(Frame::new(5)));
}

// ---------------------------------------------------------------------------
// Error rendering (middle class: diagnostics)
// ---------------------------------------------------------------------------

// CEP:WHAT: Verifies every error variant renders a non-empty, deterministic message.
// CEP:WHY: Diagnostics must be stable for kernel logs (design.md 3.1) without allocation.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_error_display_stable_and_nonempty() {
    let samples = [
        PmmError::Exhausted,
        PmmError::InvalidArgument,
        PmmError::NotAllocated,
        PmmError::RegionInUse,
        PmmError::BitmapTooSmall,
        PmmError::Overflow,
    ];
    for error in samples {
        let rendered = error.to_string();
        assert!(!rendered.is_empty());
        // Deterministic: two renders agree byte for byte.
        assert_eq!(rendered, error.to_string());
    }
}

// ---------------------------------------------------------------------------
// Happy-path smoke (1 scenario, < 5% per standards 61.4.1)
// ---------------------------------------------------------------------------

// CEP:WHAT: Allocates one frame and frees it, verifying the accounting counter at each step.
// CEP:WHY: Happy-path smoke required as the suite's minimal valid-input coverage (standards 61.4).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_alloc_free_roundtrip() {
    let mut allocator = build(4, 1);
    let frame = allocator.alloc().unwrap();
    assert_eq!(frame.number(), BASE_FRAME);
    assert_eq!(allocator.allocated_count(), 1);
    allocator.free(frame).unwrap();
    assert_eq!(allocator.allocated_count(), 0);
}

// CEP:ID: MEM-a16
// CEP:WHAT: Verifies word-boundary crossing of find_zero and find_one polarity on a two-word bitmap.
// CEP:WHY: The scan primitives are the proof surface for alloc; their boundary behavior must be pinned independently.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-pmm; results recorded in verification/evidence/.
#[test]
fn test_bitmap_find_zero_patterns() {
    // Basic word-level search behavior backing the allocator scan.
    let mut backing = [0u64; 2];
    let mut bitmap = civ_pmm::Bitmap::new(&mut backing);
    assert_eq!(bitmap.find_zero(0, 128).unwrap(), Some(0));
    bitmap.set(0).unwrap();
    bitmap.set(1).unwrap();
    assert_eq!(bitmap.find_zero(0, 128).unwrap(), Some(2));
    // Fill word 0 completely: search crosses the word boundary in one step.
    for bit in 0..64 {
        bitmap.set(bit).unwrap();
    }
    assert_eq!(bitmap.find_zero(0, 128).unwrap(), Some(64));
    assert_eq!(bitmap.find_one(0, 128).unwrap(), Some(0));
    bitmap.clear(63).unwrap();
    // The hole at bit 63 is the first free bit; bit 0 remains the first
    // set bit found by find_one.
    assert_eq!(bitmap.find_zero(0, 128).unwrap(), Some(63));
    assert_eq!(bitmap.find_one(0, 128).unwrap(), Some(0));
}
