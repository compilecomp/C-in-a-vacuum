# test_bitmap_direct_out_of_bounds_rejected

Function ID: MEM-a17 (domain MEM)

## Tier 1: Beginner

Unit test: every bitmap primitive rejects out-of-capacity indices and zero-length runs directly, with no mutation after failures.

## Tier 2: Silicon Reality

The allocator clamps indices to the managed window, so its tests cannot reach the bitmap capacity branches; this test drives the bitmap API directly past capacity (bit 64 of a one-word bitmap, limit 65, run crossing capacity) to cover the InvalidArgument branches documented in each FAILURE field and to verify failed operations leave popcount unchanged.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: n/a.
3. Wrapping: n/a inside the one-word window.
4. Error codes: InvalidArgument classification pinned for get, set, clear, find_zero, find_one, find_zero_run, set_run.
