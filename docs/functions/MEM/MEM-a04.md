# Bitmap::find_zero

Function ID: MEM-a04 (domain MEM)

## Tier 1: Beginner

Finds the lowest free bit in a window of the bitmap, or reports that there is none.

## Tier 2: Silicon Reality

Word-parallel: a word of all ones is rejected with one load and one compare; the first non-full word yields its first zero via trailing_ones. Worst case O(limit/64) loads. Lower-bound argument: any correct lowest-zero search must load every word that can contain a candidate before the answer, and this implementation loads exactly that set. Position arithmetic uses trailing_ones on a masked word; the shift (1 << position) - 1 is safe because position < 64 by construction.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: word_index cast to usize is guarded by the capacity check, which bounds it below the slice length.
3. Wrapping: none; index arithmetic stays below limit <= capacity and capacity saturates instead of overflowing.
4. Error codes: InvalidArgument when limit exceeds capacity or from exceeds limit; Ok(None) means genuinely no free bit in the interval.
