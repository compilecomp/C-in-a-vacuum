# Bitmap::find_zero_run

Function ID: MEM-a07 (domain MEM)

## Tier 1: Beginner

Finds the lowest position where a run of consecutive free bits fits in a window, or reports that none fits.

## Tier 2: Silicon Reality

First-fit with conflict skipping: each candidate is verified with run_is_zero, and a rejected candidate advances the search past its first set bit, so no bit is re-examined as a candidate origin. Total work is O(limit/64) word loads plus O(run/64) verification per distinct candidate origin, bounded by limit + run_len bit examinations overall. Lower-bound argument: returning position p requires excluding every earlier origin, which requires loading every word up to p, plus verifying the run at p; the implementation performs exactly that work up to constant factors.

## Silent Failure Checklist

1. Fallbacks: none; Ok(None) is the specified no-fit answer, not degradation.
2. Truncation: none beyond the bounds-checked word indexing shared with find_zero.
3. Wrapping: search + run_len and candidate + run_len are checked; a non-representable window end is reported as no-fit, which is mathematically correct because the run cannot fit in the window.
4. Error codes: InvalidArgument for zero run length or structural bound violations; InvalidArgument (internal-invariant class) if the conflict lookup disagrees with the failed verification, which the Kani harness asserts unreachable.
