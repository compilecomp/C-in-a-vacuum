# CapabilityTable::allocated_popcount

Function ID: CS-a06 (domain CS)

## Tier 1: Beginner

Counts allocated slots by direct inspection, independent of the accounting counter.

## Tier 2: Silicon Reality

O(capacity) one occupancy test per slot. This is the second, differently-computed source of truth for the accounting invariant: disagreement between counter and popcount is detectable corruption, not silent drift (standards 66.1).

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; total function.
