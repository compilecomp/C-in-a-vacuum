# proof_free_validates_untrusted_input

Function ID: MEM-a11 (domain MEM)

## Tier 1: Beginner

Kani proof harness: free accepts exactly the allocated frame, rejects every other u64 value with a typed error, and never mutates state on rejection.

## Tier 2: Silicon Reality

The symbolic free argument spans the full u64 range, which is the threat-model TM-2 adversary model: every out-of-window value must fail InvalidArgument, every in-window unallocated value must fail NotAllocated, and the exact allocated value is the only Ok. State-unchanged-on-rejection is asserted against the counter on every rejection path and against the popcount on the double-free path, tying the two independent accounting sources to the same conclusion. This is the double-free extermination proof for design.md 4.5.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: the full u64 argument range includes the overflow-prone top; no panic is reachable by construction of the proof.
4. Error codes: classified exhaustively: Ok only for the allocated frame; NotAllocated or InvalidArgument otherwise; the state-unchanged popcount cross-check is asserted on the double-free path, the counter assertion on every rejection path.
