# CapabilityTable::alloc

Function ID: CS-a02 (domain CS)

## Tier 1: Beginner

Allocates a new capability with the requested rights on a target object and returns its handle. The handle is the only way to use the capability later.

## Tier 2: Silicon Reality

First-fit scan over slots, O(capacity) worst case, O(1) per entry write. The scan matches the search lower bound for an unordered free-slot list; a maintained free list would make allocation O(1) and is tracked as ticket CEP-13 pending process-manager workload data. Reused slots keep their advanced generation, so old handles cannot match.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: allocated counter is bounded by capacity (proven); slot index fits u32 by the capacity bound.
4. Error codes: TableFull when no slot is free; exhaustion exactness pinned by test_table_exhaustion_exact.
