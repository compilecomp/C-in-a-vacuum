# cep_lint check_doc_sync

Function ID: CS-a23 (domain CS)

## Tier 1: Beginner

Checks bidirectional DOC-SYNC between the configuration and the standards document plus the version contract.

## Tier 2: Silicon Reality

Configured rules need headings, documented rules need configuration, and the Version line must equal standard.version; this is the mechanical enforcement of the stale-documentation law for the enforcement layer itself.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: DOC-SYNC failures with the drift class named.
