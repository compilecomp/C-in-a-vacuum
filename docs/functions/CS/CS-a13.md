# test_derive_full_space_never_escalates

Function ID: CS-a13 (domain CS)

## Tier 1: Beginner

Unit test: exhaustive sweep over every (parent rights, request rights) pair of the five named rights, with and without the grant flag, asserting the classification and child contents for all 64 x 32 combinations.

## Tier 2: Silicon Reality

64 parents (32 rights masks x grant toggle) x 32 request masks = 2048 classification checks; every successful child is verified to equal the request and sit inside the parent set (object inheritance is verified by the Kani escalation harness and test_derive_attenuates). This is the unit-test counterpart of the Kani escalation proof: the named-rights space is checked exhaustively rather than sampled, so a masking regression in any single right bit cannot hide.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: n/a.
3. Wrapping: n/a inside the named-rights space.
4. Error codes: Escalation/GrantDenied classification pinned for every refused pair.
