# test_bitmap_find_zero_patterns

Function ID: MEM-a16 (domain MEM)

## Tier 1: Beginner

Unit test: pins word-boundary crossing of find_zero and find_one polarity on a two-word bitmap.

## Tier 2: Silicon Reality

The scan primitives are the proof surface for alloc and reserve; this test pins their behavior independently of the allocator: first-zero at 0, at 2 after filling two bits, at 64 exactly at the word boundary when word zero saturates, and the hole at 63 after a clear, with find_one polarity confirmed against first-set-bit semantics. Boundary patterns here localize scan regressions to the bitmap rather than the allocator state machine.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: n/a.
3. Wrapping: n/a inside the two-word window.
4. Error codes: n/a; test artifact.
