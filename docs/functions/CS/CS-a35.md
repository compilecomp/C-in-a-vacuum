# cep_lint check_numeric_literals

Function ID: CS-a35 (domain CS)

## Tier 1: Beginner

Flags numeric literals outside the allowed set without a sanctioned context.

## Tier 2: Silicon Reality

Exemption contexts are configured patterns (const, static, enum, struct, attributes, comments, ranges); non-exempt literals outside the allow list are flagged per line.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: literals beyond i64 are skipped by the parse guard rather than wrapped.
4. Error codes: n/a; findings only.
