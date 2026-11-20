# cep_lint check_function_id

Function ID: CS-a21 (domain CS)

## Tier 1: Beginner

Checks that functions over the statement threshold declare a registered-format CEP:ID.

## Tier 2: Silicon Reality

The mechanical form of the Rosetta DOC-ID rule: more than max_id_statements statements require an ID matching the configured domain pattern, checked against the same statement counter as the block rule.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; findings only.
