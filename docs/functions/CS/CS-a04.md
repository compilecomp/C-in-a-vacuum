# CapabilityTable::revoke

Function ID: CS-a04 (domain CS)

## Tier 1: Beginner

Permanently closes the capability named by a handle. After revocation the same handle never resolves again, even if the slot is reused for a new capability.

## Tier 2: Silicon Reality

O(1) after the three fail-closed checks: generation advance (wrapping), rights and object scrub, occupancy clear. The generation advance is what keeps outstanding copies of the handle closed after slot reuse; the u32 wrap residual after 2^32 revocations of one slot is recorded in threat model TM-3 rather than silently assumed.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: generation uses wrapping_add; the wrap residual is documented (Law 2), not hidden.
4. Error codes: InvalidSlot, SlotEmpty, StaleHandle; no-mutation on every rejection pinned by test_revoke_state_unchanged_on_error.
