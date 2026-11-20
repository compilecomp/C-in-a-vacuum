# proof_operations_are_transactional

Function ID: CS-a15 (domain CS)

## Tier 1: Beginner

Kani proof harness: every fallible capability operation preserves the complete table state on error, per the CEP-SMT-01 transactionality profile.

## Tier 2: Silicon Reality

The state comparison covers all slot entries (rights, object, generation, occupancy) and the accounting counter. Symbolic operations (alloc, derive, revoke) run over symbolic handles and rights; each Err result asserts full state equality.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: none inside the modeled space.
4. Error codes: n/a; proof artifact (state-equality assertions).
