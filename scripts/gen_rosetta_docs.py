#!/usr/bin/env python3
# CEP:FILE: scripts/gen_rosetta_docs.py
# CEP:WHAT: Generates the Rosetta dual-tier function documentation and the registry manifest from a structured content table.
# CEP:WHY: Standards 68.3 requires every registered function to carry Tier 1, Tier 2, and Silent Failure Checklist sections; generating from one table keeps registry.json and docs/functions/ in exact agreement (CEP-LINT-DOC-FILE) and makes doc updates ship with the code change that caused them (doc standards 5.1).
# CEP:CLASS: CEP-2
# CEP:STATUS: complete
# CEP:FAILURE: Exits non-zero if a registered function lacks content or a target file exists outside docs/functions.
# CEP:ASSUMES: The content table in this file is updated in the same commit as the function it documents; output paths are docs/functions/<DOMAIN>/<ID>.md.
# CEP:COST: offline tool; runtime cost irrelevant.
# CEP:EVIDENCE: cep_lint CEP-LINT-DOC-FILE / DOC-TIERS / DOC-INQUISITION gates over the generated tree.
# CEP:SECURITY: reads and writes repository-trusted paths only; no network.
"""Deterministic Rosetta documentation generator (CEP-2 tooling).

Content lives in the ENTRIES table below. Running this script rewrites
docs/registry.json and docs/functions/<DOMAIN>/<ID>.md from the table, so
the registry and the doc tree cannot drift apart.
"""
import json
import os
import sys

# (ID, domain, one-line summary, tier1, tier2, silent_failures)
ENTRIES = [
 ("MEM-a01", "MEM", "FrameAllocator::new",
  "Builds a frame allocator over an aligned base address and a caller-provided bitmap backing store, and hands it back empty. Give it a base address, a frame count, and a slice of u64 words; it either returns a working allocator or a typed error.",
  "Construction performs O(1) validation (alignment predicate, nonzero count, backing capacity via div_ceil, window-top overflow checks in frame-number and byte-address space) followed by one O(words) clearing pass over the backing store. The clear is a lower bound, not overhead: caller-supplied backing contents are unconstrained, so any deterministic fresh state requires writing every word once. Failed validation writes nothing; clear_all runs only after all checks pass.",
  {"Fallbacks": "none; construction either succeeds or returns an error with no partial state.",
   "Truncation": "backing.len() and required word count are compared as u64 after widening; usize::try_from rejects counts beyond usize range with Overflow instead of truncating.",
   "Wrapping": "base_frame + frame_count and window_top * FRAME_SIZE_BYTES are both checked; Overflow is returned rather than wrapped.",
   "Error codes": "InvalidArgument, BitmapTooSmall, Overflow; each classification is pinned by a unit test and the Kani construction harness."}),
 ("MEM-a02", "MEM", "FrameAllocator::alloc",
  "Hands out one free physical frame. Call it repeatedly to get distinct frames until the window is used up, then it reports exhaustion. Freeing a frame makes it the next one returned.",
  "Amortized O(1): the rotating scan hint lands on the most recently freed hole, so the common case is one word load, one bit set, one counter increment. Worst case is O(count/64) word loads when the post-hint region is full, followed by a wrap search over the prefix; the wrap re-loads prefix words once, a documented correctness cost of the flat bitmap (ticket CEP-11 tracks the cyclic cursor alternative). Frame-number reconstruction uses checked addition; the proof harness covers uniqueness across the full bound.",
  {"Fallbacks": "the wrap search is the slow path after a failed hint-segment scan; it is explicit control flow, not silent degradation.",
   "Truncation": "none; indices are u64 throughout, and the usize cast for word indexing is bounds-checked by capacity.",
   "Wrapping": "allocated counter cannot exceed frame_count (invariant proven); base_frame + index uses checked_add mapped to Overflow.",
   "Error codes": "Exhausted when both scan segments find no zero bit; InvalidArgument only if an internal bound is violated, which the Kani harness asserts unreachable."}),
 ("MEM-a03", "MEM", "FrameAllocator::reserve_range",
  "Marks a range of frames as used without handing them out, for kernel-reserved regions. If any frame in the range is already allocated, the whole reservation is rejected and nothing changes.",
  "Two passes over the range: O(len/64) conflict scan (find_one for any set bit) then O(len/64) run set. The conflict pass runs entirely before the first write, so a rejected reservation cannot leave a half-marked region. A complete reserve must inspect every word of the range before writing any of it, so the two-pass structure matches the lower bound for an atomic reserve.",
  {"Fallbacks": "none; conflict rejection is the specified behavior, not a degraded path.",
   "Truncation": "none; range endpoints are u64 frame numbers compared against the validated window top.",
   "Wrapping": "end_index - start_index is safe because containment is checked first; allocated += len stays within frame_count by the same containment.",
   "Error codes": "InvalidArgument for empty or out-of-window ranges; RegionInUse when any overlapped bit is set, verified atomic by the reserve harness."}),
 ("MEM-a04", "MEM", "Bitmap::find_zero",
  "Finds the lowest free bit in a window of the bitmap, or reports that there is none.",
  "Word-parallel: a word of all ones is rejected with one load and one compare; the first non-full word yields its first zero via trailing_ones. Worst case O(limit/64) loads. Lower-bound argument: any correct lowest-zero search must load every word that can contain a candidate before the answer, and this implementation loads exactly that set. Position arithmetic uses trailing_ones on a masked word; the shift (1 << position) - 1 is safe because position < 64 by construction.",
  {"Fallbacks": "none.",
   "Truncation": "word_index cast to usize is guarded by the capacity check, which bounds it below the slice length.",
   "Wrapping": "none; index arithmetic stays below limit <= capacity and capacity saturates instead of overflowing.",
   "Error codes": "InvalidArgument when limit exceeds capacity or from exceeds limit; Ok(None) means genuinely no free bit in the interval."}),
 ("MEM-a05", "MEM", "Bitmap::find_one",
  "Finds the lowest allocated bit in a window of the bitmap, or reports that there is none.",
  "Mirror of find_zero with reversed polarity: zero words are skipped in one load, the first non-zero word yields its first set bit via trailing_zeros, masked so bits below the start position cannot match. Same O(limit/64) worst case and the same load-count lower bound. This routine is the conflict detector for atomic reservations.",
  {"Fallbacks": "none.",
   "Truncation": "identical bounds-checked word indexing as find_zero.",
   "Wrapping": "none.",
   "Error codes": "InvalidArgument for limit above capacity or from above limit; Ok(None) means no set bit in the interval."}),
 ("MEM-a06", "MEM", "Bitmap::run_is_zero",
  "Checks that every bit in a run of consecutive positions is free, in word-sized pieces.",
  "O(run/64 + 1) masked word tests: each word contributes one load and one AND against the window mask take_mask << position. The window shift is safe because take <= 64 - position keeps the highest window bit at or below bit 63. This routine mechanically enforces the set_run precondition (standards Law 3: debug assertion, not comment) and is the verification oracle for find_zero_run agreement.",
  {"Fallbacks": "none.",
   "Truncation": "take = min(span_in_word, remaining) is a value-level clamp of the scan window, not a data truncation.",
   "Wrapping": "start + run_len is checked; Overflow is returned rather than wrapped.",
   "Error codes": "InvalidArgument when the run crosses capacity; Overflow when the run length itself overflows u64 addition."}),
 ("MEM-a07", "MEM", "Bitmap::find_zero_run",
  "Finds the lowest position where a run of consecutive free bits fits in a window, or reports that none fits.",
  "First-fit with conflict skipping: each candidate is verified with run_is_zero, and a rejected candidate advances the search past its first set bit, so no bit is re-examined as a candidate origin. Total work is O(limit/64) word loads plus O(run/64) verification per distinct candidate origin, bounded by limit + run_len bit examinations overall. Lower-bound argument: returning position p requires excluding every earlier origin, which requires loading every word up to p, plus verifying the run at p; the implementation performs exactly that work up to constant factors.",
  {"Fallbacks": "none; Ok(None) is the specified no-fit answer, not degradation.",
   "Truncation": "none beyond the bounds-checked word indexing shared with find_zero.",
   "Wrapping": "search + run_len and candidate + run_len are checked; a non-representable window end is reported as no-fit, which is mathematically correct because the run cannot fit in the window.",
   "Error codes": "InvalidArgument for zero run length or structural bound violations; InvalidArgument (internal-invariant class) if the conflict lookup disagrees with the failed verification, which the Kani harness asserts unreachable."}),
 ("MEM-a08", "MEM", "Bitmap::set_run",
  "Marks a consecutive run of bits as allocated, flipping whole words at a time.",
  "O(run/64 + 1) read-modify-write word operations, one per touched word: each contributes one load, one OR with the window mask, one store. Lower-bound argument: flipping n bits spanning w words needs at least w RMW operations; the implementation performs exactly one per word. The free-run precondition is enforced by a debug assertion backed by run_is_zero, exercised by Kani and test builds, and documented as a caller contract in the CEP block.",
  {"Fallbacks": "none.",
   "Truncation": "none; the mask construction covers exactly [start, start + run_len).",
   "Wrapping": "start + run_len checked; InvalidArgument on zero-length runs and capacity crossings.",
   "Error codes": "InvalidArgument; debug builds additionally trap on a violated free-run precondition rather than corrupting state silently."}),
 ("MEM-a09", "MEM", "proof_new_rejects_bad_config_no_panic",
  "Kani proof harness: construction accepts exactly the structurally valid configurations and rejects all others, with no panic on any input inside the declared bound.",
  "Symbolic execution over the (address, count, backing-length) space within the declared bound: count spans 0..=128, backing length 0..=4 words, and the address spans the full u64 range below the window-top edge (the unrepresentable-top arm is exercised by unit test test_new_rejects_overflow_base, the declared FV-1 refinement gap for this harness). The harness asserts the biconditional between structural validity (alignment, nonzero, capacity) and Ok, which is stronger than one-directional rejection testing.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "the harness models usize conversion failure explicitly via the expected_ok predicate.",
   "Wrapping": "covered: window overflow paths are inside the symbolic space up to the declared bound.",
   "Error codes": "n/a; proof artifact (assertions, not error propagation)."}),
 ("MEM-a10", "MEM", "proof_alloc_exhaustion_and_uniqueness",
  "Kani proof harness: allocating every frame yields distinct frames, exhaustion is exact, and free-then-alloc reuses the exact freed frame.",
  "Uniqueness is checked with a seen-bitmap in the harness (1u64 << index over the bounded window), giving an O(n) witness per allocation against double allocation. The exhaustion boundary is asserted at exactly count allocations, and the reuse property is checked for a symbolic hole index, which subsumes every single-hole scenario within the bound. Single-word backing keeps the symbolic state space within Kani's solver budget.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none; index reconstruction is exact subtraction from the known base.",
   "Wrapping": "harness panics would surface any overflow inside the bound as a failed proof.",
   "Error codes": "n/a; proof artifact."}),
 ("MEM-a11", "MEM", "proof_free_validates_untrusted_input",
  "Kani proof harness: free accepts exactly the allocated frame, rejects every other u64 value with a typed error, and never mutates state on rejection.",
  "The symbolic free argument spans the full u64 range, which is the threat-model TM-2 adversary model: every out-of-window value must fail InvalidArgument, every in-window unallocated value must fail NotAllocated, and the exact allocated value is the only Ok. State-unchanged-on-rejection is asserted against the counter on every rejection path and against the popcount on the double-free path, tying the two independent accounting sources to the same conclusion. This is the double-free extermination proof for design.md 4.5.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "the full u64 argument range includes the overflow-prone top; no panic is reachable by construction of the proof.",
   "Error codes": "classified exhaustively: Ok only for the allocated frame; NotAllocated or InvalidArgument otherwise; the state-unchanged popcount cross-check is asserted on the double-free path, the counter assertion on every rejection path."}),
 ("MEM-a12", "MEM", "proof_contiguous_bounds_and_disjoint",
  "Kani proof harness: contiguous allocation returns exactly the requested run inside the window and successive contiguous allocations never overlap.",
  "Symbolic (count, run) over the bounded space; the disjointness witness is a range-disjoint predicate over two allocated runs covering the full window, which proves the stronger property that first-fit never overlaps regardless of where the first run landed. Failure classification asserts Exhausted is the only error a fresh allocator can produce for run <= count, pinning the InvalidArgument surface to structural violations only.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "range end computations inside the bound are asserted reachable without overflow.",
   "Error codes": "n/a; proof artifact."}),
 ("MEM-a13", "MEM", "proof_reserve_conflict_is_atomic",
  "Kani proof harness: reservation over an allocated frame is rejected atomically with state unchanged, and accepted reservations leave alloc nothing inside the reserved range to hand out.",
  "The symbolic range lies inside the managed window (out-of-window classification is covered by unit test test_reserve_range_out_of_window_rejected, the declared FV-1 refinement gap). The harness asserts the biconditional between genuine overlap and RegionInUse, proves state immutability on rejection via counter and popcount, and proves isolation on acceptance by draining the remaining window while asserting every returned frame lies outside the reserved range, with exhaustion only after the drain.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "range endpoints constrained inside the window by assumption; the out-of-window arm is the declared refinement gap covered by unit test.",
   "Error codes": "classified exhaustively within the modeled space."}),
 ("MEM-a14", "MEM", "proof_invariant_popcount_matches_allocated",
  "Kani proof harness: the accounting counter, the bitmap popcount, and the window-sum invariant agree after every operation in a nondeterministic operation sequence.",
  "Four nondeterministically chosen operations (alloc, free, invariant check) exercise the state machine against symbolic interleavings; the invariant is asserted after every check-step and at the final state. Because operations are chosen symbolically rather than from a fixed script, the harness covers the reachable operation space within its bound instead of one path.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "counter arithmetic is bounded by the window inside the proof.",
   "Error codes": "n/a; proof artifact."}),
 ("MEM-a15h", "MEM", "proof_bitmap_run_search_consistent",
  "Kani proof harness: find_zero_run agrees with the run_is_zero verifier and the returned origin is minimal.",
  "Three properties over the bounded single-word state space: Some(p) implies the run at p is verified free within the window; Some(p) is minimal, meaning no earlier origin hosts the run; None implies no origin in the interval hosts the run. The harness drives symbolic word values, so word-boundary masking adversarial patterns are inside the model rather than sampled. Multi-word windows are covered by the allocator unit tests; the declared single-word Kani bound is the refinement gap.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "candidate + run_len windows are checked by the implementation under test.",
   "Error codes": "Err classified to structural violations only; the harness asserts the classification."}),
 ("MEM-a16h", "MEM", "proof_bitmap_set_run_bounds",
  "Kani proof harness: set_run flips exactly the requested run and rejects structural violations without mutation.",
  "Ok(set_run) over a verified-free run raises the popcount by exactly run_len and sets every run bit; Err leaves the bitmap unchanged and, inside the bounds, implies the run was not free. The harness constrains the run inside one word so the window-shift arithmetic is exercised at position 0 and position 63 edges symbolically.",
  {"Fallbacks": "n/a; proof artifact.",
   "Truncation": "none.",
   "Wrapping": "start + run_len constrained within the word by assumption.",
   "Error codes": "Ok/Err biconditional asserted within the modeled space."}),
 ("MEM-a17", "MEM", "test_bitmap_direct_out_of_bounds_rejected",
  "Unit test: every bitmap primitive rejects out-of-capacity indices and zero-length runs directly, with no mutation after failures.",
  "The allocator clamps indices to the managed window, so its tests cannot reach the bitmap capacity branches; this test drives the bitmap API directly past capacity (bit 64 of a one-word bitmap, limit 65, run crossing capacity) to cover the InvalidArgument branches documented in each FAILURE field and to verify failed operations leave popcount unchanged.",
  {"Fallbacks": "n/a; test artifact.",
   "Truncation": "n/a.",
   "Wrapping": "n/a inside the one-word window.",
   "Error codes": "InvalidArgument classification pinned for get, set, clear, find_zero, find_one, find_zero_run, set_run."}),
 ("MEM-a18", "MEM", "test_frame_range_contains",
  "Unit test: FrameRange::contains classification at both boundaries and for an empty range.",
  "contains is the ownership predicate future page-table consumers rely on (design.md 4.2); this test pins inclusive-start, exclusive-end behavior at both edges and the empty-range answer, so the predicate cannot drift while it awaits its first kernel caller.",
  {"Fallbacks": "n/a; test artifact.",
   "Truncation": "n/a.",
   "Wrapping": "n/a.",
   "Error codes": "n/a; test artifact."}),
 ("MEM-a15", "MEM", "test_boundary_first_and_last_frame",
  "Unit test: exercises the first frame, the window top frame, and the exact exhaustion boundary of a 129-frame window spanning three words.",
  "129 is the smallest frame count that is non-multiple-of-64 and spans three words, so the third word is only 1/64 used and the last managed bit (128) is exactly position 0 of word 2, the boundary where an inclusive window-top check would escape the word. The test pins the first allocation to the base frame, walks to exact exhaustion, frees the window-top frame, and asserts the exact same frame is returned, which catches the classic last-word off-by-one class.",
  {"Fallbacks": "n/a; test artifact.",
   "Truncation": "n/a.",
   "Wrapping": "n/a inside the test window.",
   "Error codes": "n/a; test artifact."}),
 ("MEM-a16", "MEM", "test_bitmap_find_zero_patterns",
  "Unit test: pins word-boundary crossing of find_zero and find_one polarity on a two-word bitmap.",
  "The scan primitives are the proof surface for alloc and reserve; this test pins their behavior independently of the allocator: first-zero at 0, at 2 after filling two bits, at 64 exactly at the word boundary when word zero saturates, and the hole at 63 after a clear, with find_one polarity confirmed against first-set-bit semantics. Boundary patterns here localize scan regressions to the bitmap rather than the allocator state machine.",
  {"Fallbacks": "n/a; test artifact.",
   "Truncation": "n/a.",
   "Wrapping": "n/a inside the two-word window.",
   "Error codes": "n/a; test artifact."}),
]

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    docs_root = os.path.join(root, 'docs')
    registry_path = os.path.join(docs_root, 'registry.json')
    registry = {
        "registry_version": 2,
        "functions": [
            {"id": fid, "domain": domain, "summary": summary}
            for fid, domain, summary, _t1, _t2, _sf in ENTRIES
        ],
    }
    with open(registry_path, 'w', encoding='utf-8') as handle:
        json.dump(registry, handle, indent=2)
        handle.write('\n')
    for fid, domain, summary, tier1, tier2, silent in ENTRIES:
        target_dir = os.path.join(docs_root, 'functions', domain)
        os.makedirs(target_dir, exist_ok=True)
        target = os.path.join(target_dir, f"{fid}.md")
        content = (
            f"# {summary}\n\n"
            f"Function ID: {fid} (domain {domain})\n\n"
            f"## Tier 1: Beginner\n\n{tier1}\n\n"
            f"## Tier 2: Silicon Reality\n\n{tier2}\n\n"
            f"## Silent Failure Checklist\n\n"
            f"1. Fallbacks: {silent['Fallbacks']}\n"
            f"2. Truncation: {silent['Truncation']}\n"
            f"3. Wrapping: {silent['Wrapping']}\n"
            f"4. Error codes: {silent['Error codes']}\n"
        )
        with open(target, 'w', encoding='utf-8') as handle:
            handle.write(content)
    print(f"registry: {len(ENTRIES)} entries; docs written to docs/functions/")
    return 0

if __name__ == '__main__':
    sys.exit(main())
