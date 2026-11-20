# cep_lint check_secret_probe

Function ID: CS-a38 (domain CS)

## Tier 1: Beginner

Probes raw lines for secret-shaped content with suppression support.

## Tier 2: Silicon Reality

Secrets live inside string literals which the lexical pass blanks out of code, so the probe runs on the raw line; prose mentions without assignment context are excluded by the configured patterns.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; findings only.
