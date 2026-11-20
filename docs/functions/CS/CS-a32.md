# cep_lint run_one_doc_rule

Function ID: CS-a32 (domain CS)

## Tier 1: Beginner

Dispatches one document-scoped rule to its primitive for one file.

## Tier 2: Silicon Reality

Cross-file rules produce per-file findings; the scan-level dispatcher calls this entry point so finding attribution stays per-file while the check itself runs once per rule.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; dispatch only.
