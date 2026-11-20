# proof_bitmap_set_run_bounds

Function ID: MEM-a16h (domain MEM)

## Tier 1: Beginner

Kani proof harness: set_run flips exactly the requested run and rejects structural violations without mutation.

## Tier 2: Silicon Reality

Ok(set_run) over a verified-free run raises the popcount by exactly run_len and sets every run bit; Err leaves the bitmap unchanged and, inside the bounds, implies the run was not free. The harness constrains the run inside one word so the window-shift arithmetic is exercised at position 0 and position 63 edges symbolically.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: start + run_len constrained within the word by assumption.
4. Error codes: Ok/Err biconditional asserted within the modeled space.
