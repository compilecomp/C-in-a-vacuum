# Bitmap::set_run

Function ID: MEM-a08 (domain MEM)

## Tier 1: Beginner

Marks a consecutive run of bits as allocated, flipping whole words at a time.

## Tier 2: Silicon Reality

O(run/64 + 1) read-modify-write word operations, one per touched word: each contributes one load, one OR with the window mask, one store. Lower-bound argument: flipping n bits spanning w words needs at least w RMW operations; the implementation performs exactly one per word. The free-run precondition is enforced by a debug assertion backed by run_is_zero, exercised by Kani and test builds, and documented as a caller contract in the CEP block.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none; the mask construction covers exactly [start, start + run_len).
3. Wrapping: start + run_len checked; InvalidArgument on zero-length runs and capacity crossings.
4. Error codes: InvalidArgument; debug builds additionally trap on a violated free-run precondition rather than corrupting state silently.
