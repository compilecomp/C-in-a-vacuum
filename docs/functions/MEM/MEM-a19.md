# proof_operations_are_transactional

Function ID: MEM-a19 (domain MEM)

## Tier 1: Beginner

Kani proof harness: every fallible allocator operation preserves the complete allocator state on error, per the CEP-SMT-01 transactionality profile.

## Tier 2: Silicon Reality

The state comparison covers every mutable field the standard names (all bitmap words, the allocation counter, and the scan hint), not just the externally visible count. Symbolic operations (free, alloc, alloc_contiguous, reserve) run over a symbolic non-trivial state; each Err result asserts full state equality. This is the design-of-record discharge for the SOLVER-TOOLING.md section 5 allocator example.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: argument modulo keeps indices inside the window within the bound.
4. Error codes: n/a; proof artifact (state-equality assertions).
