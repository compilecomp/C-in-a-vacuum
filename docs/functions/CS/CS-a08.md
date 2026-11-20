# proof_derive_never_escalates

Function ID: CS-a08 (domain CS)

## Tier 1: Beginner

Kani proof harness: derive creates a child only when the request is a subset of the parent's rights and the parent holds the grant right; the child's rights equal the request on the parent's object.

## Tier 2: Silicon Reality

Symbolic parent rights and request span the full u64 space, with a symbolic grant flag; the harness asserts the full classification biconditional (Ok implies subset+grant with exact child contents; GrantDenied implies no grant; Escalation implies grant plus non-subset). Capability integrity (design.md 21) is thereby discharged over the bounded state space, not sampled.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: covered: rights arithmetic is total u64 inside the model.
4. Error codes: classification exhaustive within the modeled space.
