# CapabilityTable::derive

Function ID: CS-a03 (domain CS)

## Tier 1: Beginner

Creates a child capability from a parent. The child gets exactly the rights you request and the parent's target object, but only if the request is a subset of the parent's rights and the parent carries the grant right. Anything else is refused and nothing changes.

## Tier 2: Silicon Reality

The authority core: resolution (bounds + occupancy + generation), two right comparisons (GRANT, subset), then one slot write. Both security checks run before any mutation, so every refusal is atomic. The Kani harness proves the refusal/acceptance biconditional over the full u64 rights space; the unit suite sweeps all 2x2 x 2x5 named-right combinations exhaustively.

## Silent Failure Checklist

1. Fallbacks: none; refusal is the specified behavior.
2. Truncation: none.
3. Wrapping: none; right-set arithmetic is total over u64.
4. Error codes: ParentInvalid (unresolvable handle), GrantDenied (no RIGHT_GRANT), Escalation (non-subset request), TableFull (no slot); each pinned by a dedicated test and the escalation harness.
