# cep_lint Parser::parse_string

Function ID: CS-a28 (domain CS)

## Tier 1: Beginner

Parses a JSON string with escape handling and UTF-8 validation.

## Tier 2: Silicon Reality

Escape sequences decode byte-exactly into one buffer and UTF-8 is validated once at the closing quote; single-buffer accumulation prevents the escape-versus-raw split error class.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: parse errors with byte positions.
