# proof_invariant_slots_consistent

Function ID: CS-a12 (domain CS)

## Tier 1: Beginner

Kani proof harness: the accounting counter, the slot popcount, and the capacity subtraction agree after every operation in a nondeterministic sequence.

## Tier 2: Silicon Reality

Four nondeterministically chosen operations (alloc/revoke/invariant check) exercise the state machine against symbolic interleavings within the 4-slot bound; the invariant is asserted after every check-step and at the final state, mirroring the frame allocator's invariant harness discipline.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: counter bounded by capacity inside the model.
4. Error codes: n/a; proof artifact.
