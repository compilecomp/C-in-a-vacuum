# cep_lint check_unsafe_doc

Function ID: CS-a39 (domain CS)

## Tier 1: Beginner

Checks that every unsafe usage is documented by the configured field.

## Tier 2: Silicon Reality

Unsafe occurrences in function bodies require the documentation field in the enclosing block or file header; undocumented unsafe usage is a severity-1 defect per the security chapters.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; findings only.
