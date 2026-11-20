# proof_contiguous_bounds_and_disjoint

Function ID: MEM-a12 (domain MEM)

## Tier 1: Beginner

Kani proof harness: contiguous allocation returns exactly the requested run inside the window and successive contiguous allocations never overlap.

## Tier 2: Silicon Reality

Symbolic (count, run) over the bounded space; the disjointness witness is a range-disjoint predicate over two allocated runs covering the full window, which proves the stronger property that first-fit never overlaps regardless of where the first run landed. Failure classification asserts Exhausted is the only error a fresh allocator can produce for run <= count, pinning the InvalidArgument surface to structural violations only.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: range end computations inside the bound are asserted reachable without overflow.
4. Error codes: n/a; proof artifact.
