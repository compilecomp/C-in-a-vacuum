# cep_lint check_comment_regex

Function ID: CS-a33 (domain CS)

## Tier 1: Beginner

Applies the configured pattern to comment text with suppression support.

## Tier 2: Silicon Reality

Comment-marker rules operate on the comment text with configured suppression patterns; regex compilation failures produce coded bootstrap findings instead of silently scanning nothing.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-8 for bad patterns.
