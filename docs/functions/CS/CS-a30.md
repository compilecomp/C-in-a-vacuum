# cep_lint Parser::parse_object

Function ID: CS-a30 (domain CS)

## Tier 1: Beginner

Parses a JSON object into the ordered key map.

## Tier 2: Silicon Reality

Ordered keys keep rule iteration deterministic; duplicate keys are last-wins matching common tooling.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: parse errors with byte positions.
