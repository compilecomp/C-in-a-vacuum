# proof_reserve_conflict_is_atomic

Function ID: MEM-a13 (domain MEM)

## Tier 1: Beginner

Kani proof harness: reservation over an allocated frame is rejected atomically with state unchanged, and accepted reservations leave alloc nothing inside the reserved range to hand out.

## Tier 2: Silicon Reality

The symbolic range lies inside the managed window (out-of-window classification is covered by unit test test_reserve_range_out_of_window_rejected, the declared FV-1 refinement gap). The harness asserts the biconditional between genuine overlap and RegionInUse, proves state immutability on rejection via counter and popcount, and proves isolation on acceptance by draining the remaining window while asserting every returned frame lies outside the reserved range, with exhaustion only after the drain.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: range endpoints constrained inside the window by assumption; the out-of-window arm is the declared refinement gap covered by unit test.
4. Error codes: classified exhaustively within the modeled space.
