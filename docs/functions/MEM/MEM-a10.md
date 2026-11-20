# proof_alloc_exhaustion_and_uniqueness

Function ID: MEM-a10 (domain MEM)

## Tier 1: Beginner

Kani proof harness: allocating every frame yields distinct frames, exhaustion is exact, and free-then-alloc reuses the exact freed frame.

## Tier 2: Silicon Reality

Uniqueness is checked with a seen-bitmap in the harness (1u64 << index over the bounded window), giving an O(n) witness per allocation against double allocation. The exhaustion boundary is asserted at exactly count allocations, and the reuse property is checked for a symbolic hole index, which subsumes every single-hole scenario within the bound. Single-word backing keeps the symbolic state space within Kani's solver budget.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none; index reconstruction is exact subtraction from the known base.
3. Wrapping: harness panics would surface any overflow inside the bound as a failed proof.
4. Error codes: n/a; proof artifact.
