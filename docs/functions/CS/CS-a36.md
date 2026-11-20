# cep_lint check_macro_prefix

Function ID: CS-a36 (domain CS)

## Tier 1: Beginner

Checks macro_rules! names against the configured prefix pattern.

## Tier 2: Silicon Reality

The macro name is the first whitespace-delimited token after macro_rules!, so the brace belongs to the body and never leaks into the name check.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-8 for bad patterns.
