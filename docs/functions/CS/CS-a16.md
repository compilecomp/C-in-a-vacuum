# cep_lint scan_targets

Function ID: CS-a16 (domain CS)

## Tier 1: Beginner

Runs every configured rule over every first-party file under the targets and returns sorted findings.

## Tier 2: Silicon Reality

One scan pass collects all findings; the deterministic sort (file, line, rule, message) is the standards 50.1 output contract. Unreadable targets produce bootstrap findings rather than aborting the scan, so one bad file cannot hide findings elsewhere.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT findings for unreadable targets.
