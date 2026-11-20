# CapabilityTable::lookup

Function ID: CS-a05 (domain CS)

## Tier 1: Beginner

Turns a handle back into the capability's rights and target object, or fails closed. Any u64 value can be presented; only a live slot with a matching generation resolves.

## Tier 2: Silicon Reality

O(1): one bounds check, one occupancy check, one generation compare. Fail-closed resolution is the single gate every kernel authority check reduces to (design.md 9.1 validation, TM-3); the Kani harness runs the full u64 handle space through the classifier.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: slot index extraction masks the low 32 bits; the bounds check rejects anything out of range.
4. Error codes: InvalidSlot, SlotEmpty, StaleHandle; classification pinned by test_lookup_fail_closed and the handle-space harness.
