# proof_invariant_popcount_matches_allocated

Function ID: MEM-a14 (domain MEM)

## Tier 1: Beginner

Kani proof harness: the accounting counter, the bitmap popcount, and the window-sum invariant agree after every operation in a nondeterministic operation sequence.

## Tier 2: Silicon Reality

Four nondeterministically chosen operations (alloc, free, invariant check) exercise the state machine against symbolic interleavings; the invariant is asserted after every check-step and at the final state. Because operations are chosen symbolically rather than from a fixed script, the harness covers the reachable operation space within its bound instead of one path.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: counter arithmetic is bounded by the window inside the proof.
4. Error codes: n/a; proof artifact.
