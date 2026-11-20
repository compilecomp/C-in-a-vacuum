# cep_lint Parser::parse_array

Function ID: CS-a29 (domain CS)

## Tier 1: Beginner

Parses a JSON array with explicit comma structure.

## Tier 2: Silicon Reality

Array productions keep separator structure explicit so truncation is caught at the exact byte.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: parse errors with byte positions.
