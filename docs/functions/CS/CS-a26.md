# cep_lint Json::parse

Function ID: CS-a26 (domain CS)

## Tier 1: Beginner

Parses a complete JSON document into the value tree, rejecting trailing content.

## Tier 2: Silicon Reality

The recursive-descent parser is single-pass with byte-position errors; trailing content rejection catches truncated configurations instead of silently accepting them.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: integers outside i64 fail loudly rather than wrapping.
4. Error codes: parse errors with byte positions.
