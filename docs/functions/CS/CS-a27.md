# cep_lint Parser::parse_value

Function ID: CS-a27 (domain CS)

## Tier 1: Beginner

Parses one JSON value by dispatching on the first byte.

## Tier 2: Silicon Reality

Each production delegates to its dedicated parser so errors carry positions; the dispatch is total over the JSON subset the configuration uses.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: parse errors with byte positions.
