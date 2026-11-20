# CapabilityTable::alloc

Function ID: CS-a02 (domain CS)

## Tier 1: Beginner

Allocates a new capability with the requested rights on a target object and returns its handle. The handle is the only way to use the capability later.

## Tier 2: Silicon Reality

First-fit scan over slots, O(capacity) worst case, O(1) per entry write. The scan matches the search lower bound for an unordered free-slot list; a maintained free list would make allocation O(1) and is tracked as ticket CEP-13 pending process-manager workload data. Reused slots keep their advanced generation, so old handles cannot match. Capacity is bounded to u32::MAX slots at construction, making the u32 slot-index narrowing exact.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: the u32 slot-index narrowing is exact under the construction-time capacity bound (backing length <= u32::MAX, validated in new).
3. Wrapping: allocated counter is bounded by capacity (proven).
4. Error codes: TableFull when no slot is free; exhaustion exactness pinned by test_table_exhaustion_exact.
