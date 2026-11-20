# proof_handle_space_no_panic

Function ID: CS-a11 (domain CS)

## Tier 1: Beginner

Kani proof harness: lookup, rights_of, and revoke are total over the full u64 handle space; each returns a typed classification and no input panics.

## Tier 2: Silicon Reality

Symbolic handle over the full u64 space against a one-capability table; the harness asserts the typed-error disjunction for every non-live handle and the full ok path for the live handle, including post-revoke closure. This discharges threat model TM-2 fail-closed totality for the authority boundary.

## Silent Failure Checklist

1. Fallbacks: n/a; proof artifact.
2. Truncation: none.
3. Wrapping: none reachable within the model.
4. Error codes: classified exhaustively within the modeled space.
