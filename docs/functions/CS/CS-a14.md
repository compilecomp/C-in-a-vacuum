# test_handle_high_bits_forgery

Function ID: CS-a14 (domain CS)

## Tier 1: Beginner

Unit test: flipping each of the 64 bits of a live handle must fail closed.

## Tier 2: Silicon Reality

Maximum-entropy forgery sweep: every single-bit mutation of a live handle is presented to lookup, and only the unmutated handle resolves. This pins the handle encoding's bit-level integrity at unit-test speed, complementing the Kani full-space harness.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: the raw handle splits into (forged_raw & 0xFFFF_FFFF) as u32 and (forged_raw >> 32) as u32: mask-then-cast and shift-then-cast are exact, lossless halves of the u64, so all 64 single-bit flips remain distinct rejected handles.
3. Wrapping: n/a.
4. Error codes: typed rejection asserted for all 64 mutated handles (every single-bit flip differs from the live handle).
