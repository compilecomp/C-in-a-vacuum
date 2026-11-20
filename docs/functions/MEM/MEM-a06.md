# Bitmap::run_is_zero

Function ID: MEM-a06 (domain MEM)

## Tier 1: Beginner

Checks that every bit in a run of consecutive positions is free, in word-sized pieces.

## Tier 2: Silicon Reality

O(run/64 + 1) masked word tests: each word contributes one load and one AND against the window mask take_mask << position. The window shift is safe because take <= 64 - position keeps the highest window bit at or below bit 63. This routine mechanically enforces the set_run precondition (standards Law 3: debug assertion, not comment) and is the verification oracle for find_zero_run agreement.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: take = min(span_in_word, remaining) is a value-level clamp of the scan window, not a data truncation.
3. Wrapping: start + run_len is checked; Overflow is returned rather than wrapped.
4. Error codes: InvalidArgument when the run crosses capacity; Overflow when the run length itself overflows u64 addition.
