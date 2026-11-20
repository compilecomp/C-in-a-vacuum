# proof_bitmap_run_search_consistent

Function ID: MEM-a15h (domain MEM)

## Tier 1: Beginner

Kani proof harness: find_zero_run agrees with the run_is_zero verifier and the returned origin is minimal.

## Tier 2: Silicon Reality

Three properties over the bounded single-word state space: Some(p) implies the run at p is verified free within the window; Some(p) is minimal, meaning no earlier origin hosts the run; None implies no origin in the interval hosts the run. The harness drives symbolic word values, so word-boundary masking adversarial patterns are inside the model rather than sampled. Multi-word windows are covered by the allocator unit tests; the declared single-word Kani bound is the refinement gap.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: candidate + run_len windows are checked by the implementation under test.
4. Error codes: Err classified to structural violations only; the harness asserts the classification.
