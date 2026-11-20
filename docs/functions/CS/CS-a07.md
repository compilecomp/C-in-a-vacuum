# Rights::Display

Function ID: CS-a07 (domain CS)

## Tier 1: Beginner

Renders a rights set as a short fixed string naming exactly the rights held, such as read+grant or none.

## Tier 2: Silicon Reality

At most five named-right writes plus a numeric suffix for unnamed bits; no allocation. Unnamed bits render as their hex value, never silently dropped (standards 66.1), so audit logs remain faithful to the actual authority held.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none; unnamed bits are rendered, not truncated.
3. Wrapping: none.
4. Error codes: none; total over all u64 sets.
