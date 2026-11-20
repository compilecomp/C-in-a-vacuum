# cep_lint config::load

Function ID: CS-a31 (domain CS)

## Tier 1: Beginner

Loads the lint configuration from disk and validates the schema.

## Tier 2: Silicon Reality

Bootstrap validation fails with coded errors before any scan; severity ranges and required keys are checked once so check primitives consume options without re-validating.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-1/2/4/5/7 coded classes.
