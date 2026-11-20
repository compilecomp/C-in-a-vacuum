# FrameAllocator::alloc

Function ID: MEM-a02 (domain MEM)

## Tier 1: Beginner

Hands out one free physical frame. Call it repeatedly to get distinct frames until the window is used up, then it reports exhaustion. Freeing a frame makes it the next one returned.

## Tier 2: Silicon Reality

Amortized O(1): the rotating scan hint lands on the most recently freed hole, so the common case is one word load, one bit set, one counter increment. Worst case is O(count/64) word loads when the post-hint region is full, followed by a wrap search over the prefix; the wrap re-loads prefix words once, a documented correctness cost of the flat bitmap (ticket CEP-11 tracks the cyclic cursor alternative). Frame-number reconstruction uses checked addition; the proof harness covers uniqueness across the full bound.

## Silent Failure Checklist

1. Fallbacks: the wrap search is the slow path after a failed hint-segment scan; it is explicit control flow, not silent degradation.
2. Truncation: none; indices are u64 throughout, and the usize cast for word indexing is bounds-checked by capacity.
3. Wrapping: allocated counter cannot exceed frame_count (invariant proven); base_frame + index uses checked_add mapped to Overflow.
4. Error codes: Exhausted when both scan segments find no zero bit; InvalidArgument only if an internal bound is violated, which the Kani harness asserts unreachable.
