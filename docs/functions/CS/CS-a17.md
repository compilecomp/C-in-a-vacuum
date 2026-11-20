# cep_lint run_scan_level_rules

Function ID: CS-a17 (domain CS)

## Tier 1: Beginner

Runs scan-level rules once per scan: DOC-SYNC and the deferred receipt-infrastructure rules.

## Tier 2: Silicon Reality

Cross-file rules must run once, not once per file; the three receipt rules emit recorded-deviation findings per their tickets (CEP-90, CEP-91), which keeps DOC-SYNC agreement honest while the CI infrastructure is pending.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: DOC-SYNC failures and recorded-deviation findings.
