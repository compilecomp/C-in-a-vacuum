# cep_lint collect_block

Function ID: CS-a40 (domain CS)

## Tier 1: Beginner

Collects the contiguous CEP block above a function definition.

## Tier 2: Silicon Reality

Block adjacency walks up through attributes and rustdoc to the CEP run; the file-header separation rule prevents header blocks from being accepted as function blocks.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; extraction only.
