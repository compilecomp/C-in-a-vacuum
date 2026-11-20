// CEP:FILE: source/kernel/civ-pmm/src/proofs.rs
// CEP:WHAT: Kani bounded model-checking harnesses for the physical memory manager: construction validation, allocation uniqueness and exhaustion, free validation over the full u64 input range, contiguous-run bounds and disjointness, reserve atomicity, the accounting invariant, and bitmap run-search agreement plus run-set bounds.
// CEP:WHY: design.md 4.5 memory-correctness obligations (no double free, no unauthorized allocation, panic freedom over untrusted inputs) must be discharged mechanically, not by review comments (standards 54.1 FV prime law).
// CEP:CLASS: FV-1
// CEP:STATUS: complete
// CEP:FAILURE: A failing harness aborts cargo kani with a counterexample trace; a harness that fails to compile blocks CI. Harnesses are excluded from non-Kani builds by cfg(kani).
// CEP:ASSUMES: Bounded model: frame counts are bounded per harness (<= 64 or <= 128 as declared), backing stores are fixed-size arrays, per-harness unwind bounds are declared via #[kani::unwind] (4 to 72, annotated on every harness), and the frame-size constant is the x86-64 4 KiB baseline. Bounds are recorded in verification/proof-registry.json; properties beyond the bound are covered by checked arithmetic plus property tests, and this gap is the declared FV-1 refinement note.
// CEP:COST: proof-time only; cfg(kani) means zero instructions are linked into any kernel or test image.
// CEP:EVIDENCE: cargo kani -p civ-pmm results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json); registry entries in verification/proof-registry.json.
// CEP:SECURITY: Harnesses model adversarial inputs with kani::any over full u64 ranges where the threat model (security/threat_model.md TM-1, TM-2) declares untrusted inputs.
// CEP:FV-CLASS: FV-1
// CEP:FV-PROVER: Kani (exact version recorded in verification/evidence/toolchain.json at proof time)
// CEP:FV-SCRIPT: cargo kani -p civ-pmm
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: Harnesses exercise the exact kernel source functions (no abstract model). Bounded: frame count <= 128, backing arrays <= 4 words, symbolic addresses constrained below the u64 window-top edge (unit test test_new_rejects_overflow_base covers that arm), reserve ranges constrained inside the managed window (unit test test_reserve_range_out_of_window_rejected covers the rest); unbounded universality is not claimed and is the declared FV-1 gap, mitigated by runtime checked arithmetic.
// CEP:FV-AXIOMS: none
//! Kani proof harnesses for the physical memory manager.
//!
//! Every harness runs against the same source the kernel ships; there is no
//! separate abstract model to drift out of sync. Bounds are declared per
//! harness and registered in verification/proof-registry.json.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::allocator::FrameAllocator;
use crate::bitmap::Bitmap;
use crate::error::PmmError;
use crate::frame::{Frame, FrameRange, PhysAddr, FRAME_SIZE_BYTES};

/// Proof base address: frame-aligned, comfortably away from the u64 top so
/// window arithmetic inside the bound cannot overflow.
const PROOF_BASE: u64 = 0x1000_0000;
/// Frame number of PROOF_BASE.
const PROOF_BASE_FRAME: u64 = PROOF_BASE / FRAME_SIZE_BYTES;

/// Harness: construction accepts exactly the structurally valid configurations
/// and rejects everything else without panicking.
///
/// Model bound: frame count <= 128, backing length <= 4 words (256 bits).
/// The overflow arm of the rejection predicate is unreachable inside the
/// bound because PROOF_BASE_FRAME + 128 fits u64; that arm is separately
/// covered by unit test test_new_rejects_overflow_base.
// CEP:ID: MEM-a09
// CEP:WHAT: Construction accepts exactly the structurally valid configurations and rejects all others without panicking.
// CEP:WHY: Discharges TM-1/TM-2 fail-closed construction over symbolic addresses, counts, and backing lengths within the declared bound.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(8)]
fn proof_new_rejects_bad_config_no_panic() {
    let address: u64 = kani::any();
    let count: u64 = kani::any();
    kani::assume(count <= 128);
    // Model bound: the window top address must be representable, mirroring
    // the construction contract. The unbounded-top arm (window address
    // overflow) is exercised by unit test test_new_rejects_overflow_base
    // and is the declared FV-1 refinement gap for this harness.
    kani::assume(address <= u64::MAX - FRAME_SIZE_BYTES * 128);
    let backing = [0u64; 4];
    let use_len: usize = kani::any();
    kani::assume(use_len <= 4);

    let result = FrameAllocator::new(PhysAddr::new(address), count, &mut backing[..use_len]);

    // No panic on any (address, count, use_len) within the bound: reached here.
    let aligned = address % FRAME_SIZE_BYTES == 0;
    let required_words = if count == 0 {
        0
    } else {
        ((count - 1) / 64 + 1) as usize
    };
    let expected_ok = aligned && count > 0 && use_len >= required_words;
    assert_eq!(
        result.is_ok(),
        expected_ok,
        "construction must fail closed exactly on structural violations"
    );
    if let Ok(allocator) = result {
        assert_eq!(allocator.frame_count(), count);
        assert_eq!(allocator.allocated_count(), 0);
        assert_eq!(allocator.allocated_popcount(), 0);
    }
}

/// Harness: allocating every frame yields distinct frames, then the allocator
/// reports exhaustion; freeing any allocated frame makes exactly that frame
/// reusable and exhaustion returns.
///
/// Model bound: frame count <= 64 (single-word backing).
// CEP:ID: MEM-a10
// CEP:WHAT: Allocating every frame yields distinct frames, exhaustion is exact, and free-then-alloc reuses the exact freed frame.
// CEP:WHY: Discharges allocation uniqueness and exact exhaustion for design.md 4.5.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(72)]
fn proof_alloc_exhaustion_and_uniqueness() {
    let count: u64 = kani::any();
    kani::assume(count > 0 && count <= 64);
    let mut backing = [0u64; 1];
    let mut allocator =
        FrameAllocator::new(PhysAddr::new(PROOF_BASE), count, &mut backing).unwrap();

    let mut seen_bits: u64 = 0;
    let mut step = 0;
    while step < count {
        let frame = match allocator.alloc() {
            Ok(frame) => frame,
            Err(err) => panic!("alloc must succeed before exhaustion: {err:?}"),
        };
        let index = frame.number() - PROOF_BASE_FRAME;
        assert!(index < count, "allocated frame escaped the managed window");
        let bit = 1u64 << index;
        assert_eq!(seen_bits & bit, 0, "frame allocated twice");
        seen_bits |= bit;
        step += 1;
    }
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
    assert_eq!(allocator.allocated_count(), count);
    assert_eq!(allocator.allocated_popcount(), count);

    let hole: u64 = kani::any();
    kani::assume(hole < count);
    let hole_frame = Frame::new(PROOF_BASE_FRAME + hole);
    allocator.free(hole_frame).unwrap();
    assert_eq!(allocator.allocated_count(), count - 1);
    let reused = allocator.alloc().unwrap();
    assert_eq!(
        reused.number(),
        hole_frame.number(),
        "free-then-alloc must reuse the exact freed frame"
    );
    assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
}

/// Harness: free accepts exactly the allocated frame, rejects every other
/// u64 value with a typed error, and never mutates state on rejection.
///
/// Model bound: 64 frames; free input spans the full u64 range (threat
/// model TM-2 untrusted argument).
// CEP:ID: MEM-a11
// CEP:WHAT: Free accepts exactly the allocated frame, rejects every other u64 value with a typed error, and never mutates state on rejection.
// CEP:WHY: Discharges the no-double-free obligation over the full untrusted u64 input range (TM-2).
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(4)]
fn proof_free_validates_untrusted_input() {
    let mut backing = [0u64; 1];
    let mut allocator = FrameAllocator::new(PhysAddr::new(PROOF_BASE), 64, &mut backing).unwrap();
    let allocated = allocator.alloc().unwrap();
    let before = allocator.allocated_count();

    let target: u64 = kani::any();
    let result = allocator.free(Frame::new(target));

    if target == allocated.number() {
        assert!(result.is_ok(), "freeing the allocated frame must succeed");
        assert_eq!(allocator.allocated_count(), before - 1);
        assert!(matches!(
            allocator.free(Frame::new(target)),
            Err(PmmError::NotAllocated),
            "double free must be rejected"
        ));
        assert_eq!(
            allocator.allocated_count(),
            before - 1,
            "double free must not mutate"
        );
        assert_eq!(
            allocator.allocated_popcount(),
            before - 1,
            "bitmap and accounting must agree after rejection"
        );
    } else {
        assert!(
            matches!(
                result,
                Err(PmmError::InvalidArgument) | Err(PmmError::NotAllocated)
            ),
            "free of any other u64 value must fail closed with a typed error"
        );
        assert_eq!(
            allocator.allocated_count(),
            before,
            "rejected free must not mutate"
        );
    }
}

/// Harness: contiguous allocation returns exactly the requested run inside
/// the window, and a second contiguous request cannot overlap the first.
///
/// Model bound: frame count <= 64, run length <= frame count.
// CEP:ID: MEM-a12
// CEP:WHAT: Contiguous allocation returns exactly the requested run inside the window and successive runs never overlap.
// CEP:WHY: Discharges the no-overlap obligation for contiguous requests over symbolic count and run length.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(72)]
fn proof_contiguous_bounds_and_disjoint() {
    let count: u64 = kani::any();
    kani::assume(count > 0 && count <= 64);
    let run: u64 = kani::any();
    kani::assume(run > 0 && run <= count);
    let mut backing = [0u64; 1];
    let mut allocator =
        FrameAllocator::new(PhysAddr::new(PROOF_BASE), count, &mut backing).unwrap();

    match allocator.alloc_contiguous(run) {
        Ok(range) => {
            assert_eq!(range.len(), run);
            assert!(range.start().number() >= PROOF_BASE_FRAME);
            assert!(range.end().number() <= PROOF_BASE_FRAME + count);
            assert_eq!(allocator.allocated_count(), run);
            assert_eq!(allocator.allocated_popcount(), run);
            if count > run {
                match allocator.alloc_contiguous(count - run) {
                    Ok(rest) => {
                        let disjoint = rest.end().number() <= range.start().number()
                            || range.end().number() <= rest.start().number();
                        assert!(disjoint, "contiguous allocations overlapped");
                        assert_eq!(allocator.allocated_count(), count);
                    }
                    Err(err) => panic!("remaining run must be allocatable: {err:?}"),
                }
            } else {
                assert!(matches!(
                    allocator.alloc_contiguous(1),
                    Err(PmmError::Exhausted)
                ));
            }
        }
        Err(err) => {
            assert!(
                matches!(err, PmmError::Exhausted),
                "an empty allocator always hosts any run <= count: {err:?}"
            );
            assert_eq!(allocator.allocated_count(), 0);
        }
    }
}

/// Harness: reservation over a range that intersects an allocation is
/// rejected atomically with state unchanged, and accepted reservations are
/// never handed out by alloc.
///
/// Model bound: 64 frames; the reserved range lies inside the managed window
/// (out-of-window classification is covered by unit test
/// test_reserve_range_out_of_window_rejected, the declared FV-1 refinement
/// gap for this harness).
// CEP:ID: MEM-a13
// CEP:WHAT: Reservation over an allocation is rejected atomically with state unchanged; accepted reservations leave alloc nothing free.
// CEP:WHY: Discharges reserve atomicity and the reserved-memory isolation property.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(72)]
fn proof_reserve_conflict_is_atomic() {
    let mut backing = [0u64; 1];
    let mut allocator = FrameAllocator::new(PhysAddr::new(PROOF_BASE), 64, &mut backing).unwrap();
    let allocated = allocator.alloc().unwrap();
    let before = allocator.allocated_count();

    let start: u64 = kani::any();
    let end: u64 = kani::any();
    kani::assume(start >= PROOF_BASE_FRAME);
    kani::assume(start < end);
    kani::assume(end <= PROOF_BASE_FRAME + 64);
    let range = FrameRange::new(Frame::new(start), Frame::new(end)).unwrap();

    let covers_allocated = start <= allocated.number() && allocated.number() < end;
    match allocator.reserve_range(range) {
        Err(PmmError::RegionInUse) => {
            assert!(covers_allocated, "RegionInUse must imply an overlap");
            assert_eq!(
                allocator.allocated_count(),
                before,
                "rejected reserve must not mutate"
            );
            assert_eq!(allocator.allocated_popcount(), before);
        }
        Ok(()) => {
            assert!(
                !covers_allocated,
                "reserve succeeded over an allocated frame"
            );
            assert_eq!(allocator.allocated_count(), before + range.len());
            // Drain the remaining window: every returned frame must lie
            // outside the reserved range, and exhaustion must arrive only
            // after the drain is complete.
            let remaining = 64 - (before + range.len());
            let mut drained = 0;
            while drained < remaining {
                let frame = match allocator.alloc() {
                    Ok(frame) => frame,
                    Err(err) => panic!("free frames must remain allocatable: {err:?}"),
                };
                let outside = frame.number() < start || frame.number() >= end;
                assert!(outside, "alloc returned a frame inside the reserved range");
                drained += 1;
            }
            assert!(matches!(allocator.alloc(), Err(PmmError::Exhausted)));
        }
        Err(other) => panic!("unexpected error class: {other:?}"),
    }
}

/// Harness: the accounting counter and the bitmap popcount agree after every
/// valid operation sequence (crate invariant 2), and the counter never
/// exceeds the window.
///
/// Model bound: 64 frames; four nondeterministically chosen operations.
// CEP:ID: MEM-a14
// CEP:WHAT: The accounting counter, the bitmap popcount, and the window-sum invariant agree after every operation in a nondeterministic sequence.
// CEP:WHY: Discharges crate invariant 2 under arbitrary operation interleavings within the bound.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(12)]
fn proof_invariant_popcount_matches_allocated() {
    let mut backing = [0u64; 1];
    let mut allocator = FrameAllocator::new(PhysAddr::new(PROOF_BASE), 64, &mut backing).unwrap();
    let mut last_allocated: Option<Frame> = None;

    let mut step = 0;
    while step < 4 {
        let operation: u8 = kani::any();
        match operation % 3 {
            0 => {
                if let Ok(frame) = allocator.alloc() {
                    last_allocated = Some(frame);
                }
            }
            1 => {
                if let Some(frame) = last_allocated {
                    assert!(allocator.free(frame).is_ok());
                    last_allocated = None;
                }
            }
            _ => {
                assert_eq!(
                    allocator.allocated_count(),
                    allocator.allocated_popcount(),
                    "accounting drifted from bitmap"
                );
                assert!(allocator.allocated_count() <= allocator.frame_count());
                assert_eq!(
                    allocator.allocated_count() + allocator.free_count(),
                    allocator.frame_count()
                );
            }
        }
        step += 1;
    }
    assert_eq!(
        allocator.allocated_count(),
        allocator.allocated_popcount(),
        "final state drifted from bitmap"
    );
}

/// Harness: find_zero_run agreement with the run_is_zero verifier and
/// minimality of the returned origin.
///
/// Property 1: a Some(p) result implies the run at p is verified free and
/// lies within [from, limit). Property 2: a Some(p) result is minimal; no
/// earlier origin hosts the run. Property 3: a None result implies no
/// origin in the interval hosts the run. Together these pin first-fit
/// completeness (the property alloc_contiguous relies on) for the bounded
/// single-word state space, exercising word-boundary masking adversarially.
///
/// Model bound: one bitmap word (64 bits); run length 1..=64; window within
/// the word. Multi-word windows are covered by the allocator unit tests (the
/// declared single-word Kani bound is the refinement gap for this harness).
// CEP:ID: MEM-a15h
// CEP:WHAT: Kani harness proving find_zero_run agreement with run_is_zero and minimality of the returned origin over the bounded single-word space.
// CEP:WHY: First-fit completeness is the property alloc_contiguous relies on; pinning it against the independent verifier on symbolic word values puts the adversarial masking patterns inside the model instead of sampling them.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: One bitmap word (64 bits); run length 1..=64; window within the word; multi-word windows are the declared FV-1 refinement gap covered by the allocator unit tests.
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(72)]
fn proof_bitmap_run_search_consistent() {
    let word: u64 = kani::any();
    let mut backing = [word];
    let bitmap = Bitmap::new(&mut backing);

    let from: u64 = kani::any();
    let run_len: u64 = kani::any();
    let limit: u64 = kani::any();
    kani::assume(from <= limit && limit <= 64);
    kani::assume(run_len >= 1 && run_len <= 64);

    let result = bitmap.find_zero_run(from, run_len, limit);
    match result {
        Ok(Some(origin)) => {
            assert!(origin >= from, "origin precedes the search start");
            assert!(origin + run_len <= limit, "run escapes the window");
            assert!(
                matches!(bitmap.run_is_zero(origin, run_len), Ok(true)),
                "returned run is not free"
            );
            let mut earlier = from;
            while earlier < origin {
                assert!(
                    !matches!(bitmap.run_is_zero(earlier, run_len), Ok(true)),
                    "an earlier origin hosts the run: minimality violated"
                );
                earlier += 1;
            }
        }
        Ok(None) => {
            let mut candidate = from;
            while candidate + run_len <= limit {
                assert!(
                    !matches!(bitmap.run_is_zero(candidate, run_len), Ok(true)),
                    "find_zero_run reported None despite a free run"
                );
                candidate += 1;
            }
        }
        Err(_) => {
            assert!(
                run_len == 0 || from > limit,
                "Err requires a structural violation"
            );
        }
    }
}

/// Harness: set_run flips exactly the requested run, requires the run free,
/// and rejects structural violations.
///
/// Property 1: Ok(set_run) over a verified-free run raises the popcount by
/// exactly run_len and sets every bit of the run. Property 2: zero-length
/// and out-of-bounds runs are rejected with InvalidArgument and leave the
/// bitmap unchanged. Property 3: the debug precondition (run free) is
/// discharged by construction: the harness only calls set_run over runs it
/// verified free with run_is_zero.
///
/// Model bound: one bitmap word (64 bits).
// CEP:ID: MEM-a16h
// CEP:WHAT: Kani harness proving set_run flips exactly the requested run and rejects structural violations without mutation.
// CEP:WHY: set_run's exact-flip contract backs reserve_range and alloc_contiguous accounting; the Ok/Err biconditional over symbolic word values covers the position-0 and position-63 masking edges symbolically.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: One bitmap word (64 bits); run in 1..=64 within bounds; the free-run precondition is established by run_is_zero inside the harness before every set_run call.
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(72)]
fn proof_bitmap_set_run_bounds() {
    let word: u64 = kani::any();
    let mut backing = [word];
    let mut bitmap = Bitmap::new(&mut backing);

    let start: u64 = kani::any();
    let run_len: u64 = kani::any();
    kani::assume(run_len >= 1 && run_len <= 64);
    // Overflow-safe bound: 64 - run_len >= 0, so the subtraction cannot
    // wrap and start + run_len <= 64 holds without evaluating an unchecked
    // addition on a symbolic operand.
    kani::assume(start <= 64 - run_len);

    let free_before = matches!(bitmap.run_is_zero(start, run_len), Ok(true));
    let popcount_before = bitmap.popcount();
    if free_before {
        // set_run's contract over verified-free runs: exact flip.
        bitmap.set_run(start, run_len).unwrap();
        assert_eq!(bitmap.popcount(), popcount_before + run_len);
        let mut bit = start;
        while bit < start + run_len {
            assert!(matches!(bitmap.get(bit), Ok(true)));
            bit += 1;
        }
    } else {
        // set_run's precondition is a verified-free run, enforced by a
        // debug assertion that traps on violation; the harness therefore
        // must not call it over a non-free run. Structural rejections are
        // covered by the concrete calls below.
        assert!(matches!(
            bitmap.set_run(0, 0),
            Err(PmmError::InvalidArgument)
        ));
        assert!(matches!(
            bitmap.set_run(64, 1),
            Err(PmmError::InvalidArgument)
        ));
        assert!(matches!(
            bitmap.set_run(u64::MAX, 1),
            Err(PmmError::Overflow)
        ));
        assert_eq!(bitmap.popcount(), popcount_before, "failed set_run mutated");
    }
}

/// Harness: transactionality of every fallible allocator operation
/// (CEP-SMT-01 PROFILE: TRANSACTIONALITY, section 5 allocator example).
///
/// Property (CEP-SMT-01 2.1/5): for every fallible operation,
/// `R = Err => S1 == S0` where the state comparison covers EVERY mutable
/// field relevant to the contract: all bitmap words, the allocation
/// counter, and the scan hint. Checking only the externally visible
/// allocation count is insufficient per section 2.1.
///
/// Model bound: 64 frames (single-word backing), four nondeterministically
/// chosen operations over symbolic arguments.
// CEP:ID: MEM-a19
// CEP:WHAT: Kani harness proving transactionality of every fallible allocator operation per the CEP-SMT-01 transactionality profile.
// CEP:WHY: SOLVER-TOOLING.md section 5 names this exact property for FrameAllocator; the state comparison covers all bitmap words, the counter, and the scan hint as the standard requires, discharging the failed-operation-preserves-state obligation mechanically.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Bounded model: 64 frames, single-word backing, three symbolic operations over symbolic arguments (see verification/smt/ reports).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-pmm; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json); report in verification/smt/MEM-a19.report.md.
#[kani::proof]
#[kani::unwind(72)]
fn proof_operations_are_transactional() {
    let count: u64 = kani::any();
    kani::assume(count > 0 && count <= 64);
    let mut backing = [0u64; 1];
    let mut allocator =
        FrameAllocator::new(PhysAddr::new(PROOF_BASE), count, &mut backing).unwrap();
    // Establish a non-trivial symbolic state: allocate two frames and free
    // one, so the bitmap, counter, and hint are all exercised.
    let first = allocator.alloc().unwrap();
    let second = allocator.alloc().unwrap();
    allocator.free(first).unwrap();

    let mut step = 0;
    while step < 3 {
        let snapshot_words = allocator.bitmap.words[0];
        let snapshot_allocated = allocator.allocated;
        let snapshot_hint = allocator.scan_hint;

        let operation: u8 = kani::any();
        let argument: u64 = kani::any();
        let result = match operation % 4 {
            0 => allocator
                .free(Frame::new(PROOF_BASE_FRAME + (argument % count.max(1))))
                .map(|_| ()),
            1 => allocator.alloc().map(|frame| frame.number()),
            2 => allocator
                .alloc_contiguous((argument % count.max(1)).max(1))
                .map(|range| range.len()),
            _ => allocator
                .reserve_range(
                    FrameRange::new(
                        Frame::new(PROOF_BASE_FRAME + (argument % count.max(1))),
                        Frame::new(PROOF_BASE_FRAME + count),
                    )
                    .unwrap(),
                )
                .map(|_| ()),
        };
        if result.is_err() {
            // CEP-SMT-01 transactionality predicate over the complete state.
            assert_eq!(
                allocator.bitmap.words[0], snapshot_words,
                "bitmap mutated on Err"
            );
            assert_eq!(
                allocator.allocated, snapshot_allocated,
                "counter mutated on Err"
            );
            assert_eq!(
                allocator.scan_hint, snapshot_hint,
                "scan hint mutated on Err"
            );
        }
        let _ = second;
        step += 1;
    }
}
