# cep_lint check_name_validation

Function ID: CS-a22 (domain CS)

## Tier 1: Beginner

Validates identifiers located by the detection pattern against the naming pattern.

## Tier 2: Silicon Reality

The detection pattern locates the construct (struct, enum, union) and the name pattern validates the extracted identifier; two-pattern composition keeps the type-naming rule data-driven without capture-group support in the matcher.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: the identifier extraction takes the token after the keyword, which is exact for whitespace-separated declarations.
3. Wrapping: none.
4. Error codes: n/a; findings only.
