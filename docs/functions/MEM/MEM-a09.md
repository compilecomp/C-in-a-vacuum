# proof_new_rejects_bad_config_no_panic

Function ID: MEM-a09 (domain MEM)

## Tier 1: Beginner

Kani proof harness: construction accepts exactly the structurally valid configurations and rejects all others, with no panic on any input inside the declared bound.

## Tier 2: Silicon Reality

Symbolic execution over the (address, count, backing-length) space within the declared bound: count spans 0..=128, backing length 0..=4 words, and the address spans the full u64 range below the window-top edge (the unrepresentable-top arm is exercised by unit test test_new_rejects_overflow_base, the declared FV-1 refinement gap for this harness). The harness asserts the biconditional between structural validity (alignment, nonzero, capacity) and Ok, which is stronger than one-directional rejection testing.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: the harness models usize conversion failure explicitly via the expected_ok predicate.
3. Wrapping: covered: window overflow paths are inside the symbolic space up to the declared bound.
4. Error codes: n/a; proof artifact (assertions, not error propagation).
