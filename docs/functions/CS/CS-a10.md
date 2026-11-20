# proof_revoke_closes_access

Function ID: CS-a10 (domain CS)

## Tier 1: Beginner

Kani proof harness: after a successful revoke the handle never resolves again, re-revocation is rejected, and slot reuse leaves the old handle closed.

## Tier 2: Silicon Reality

The harness walks the full lifecycle: alloc, resolve, revoke, verify closure, double-revoke rejection, slot reuse, generation advance, stale-handle closure, and live new-handle resolution. The generation-advance mechanism (the reason reuse cannot revive old handles) is thereby exercised end to end within the bound.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: generation advance stays inside u32 within the model; the 2^32 wrap residual is the declared gap.
4. Error codes: classified exhaustively within the modeled space.
