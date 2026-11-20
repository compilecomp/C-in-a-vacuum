# cep_lint check_function_block

Function ID: CS-a20 (domain CS)

## Tier 1: Beginner

Checks every nontrivial function for a complete CEP comment block.

## Tier 2: Silicon Reality

Functions with at least min_body_statements need the seven-field block; the statement threshold is the linter-consistent counter, so the check and the DOC-ID rule agree on what nontrivial means.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; findings only.
