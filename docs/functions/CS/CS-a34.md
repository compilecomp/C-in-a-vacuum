# cep_lint check_string_regex

Function ID: CS-a34 (domain CS)

## Tier 1: Beginner

Applies the configured pattern to string-literal contents.

## Tier 2: Silicon Reality

String contents are extracted from raw lines with escape awareness so placeholder-text rules see exactly what would reach runtime messages.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-8 for bad patterns.
