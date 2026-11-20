# cep_lint run_one_rule

Function ID: CS-a18 (domain CS)

## Tier 1: Beginner

Dispatches one rule to its configured check primitive with severity and options bound.

## Tier 2: Silicon Reality

The single dispatch point keeps primitive signatures uniform; unknown rule and mode pairs are skipped silently, which the self-test manifest catches because every rule has a scenario with an exact expected count.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; dispatch only.
