# cep_lint run_self_test

Function ID: CS-a25 (domain CS)

## Tier 1: Beginner

Runs the self-test manifest with exact expected finding counts per scenario.

## Tier 2: Silicon Reality

Every manifest scenario must match its expected count; a zero-failure run is the self-verification gate for the tool itself before it may judge anything else.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; exit-code contract.
