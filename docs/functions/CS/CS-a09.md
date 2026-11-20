# proof_lookup_fail_closed

Function ID: CS-a09 (domain CS)

## Tier 1: Beginner

Kani proof harness: lookup resolves exactly the live handles and fails closed with a typed error for every other u64 value, with no panic anywhere in the handle space.

## Tier 2: Silicon Reality

Symbolic handle over the full u64 space against a two-capability table with symbolic contents; the harness classifies every input into resolve-correctly or typed-rejection and verifies returned contents match the allocated capability exactly. This is the TM-3 forgery discharge.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: none reachable: extraction masks are total.
4. Error codes: classification exhaustive within the modeled space.
