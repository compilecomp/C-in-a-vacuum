# Bitmap::find_one

Function ID: MEM-a05 (domain MEM)

## Tier 1: Beginner

Finds the lowest allocated bit in a window of the bitmap, or reports that there is none.

## Tier 2: Silicon Reality

Mirror of find_zero with reversed polarity: zero words are skipped in one load, the first non-zero word yields its first set bit via trailing_zeros, masked so bits below the start position cannot match. Same O(limit/64) worst case and the same load-count lower bound. This routine is the conflict detector for atomic reservations.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: identical bounds-checked word indexing as find_zero.
3. Wrapping: none.
4. Error codes: InvalidArgument for limit above capacity or from above limit; Ok(None) means no set bit in the interval.
