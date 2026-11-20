# CapabilityTable::new

Function ID: CS-a01 (domain CS)

## Tier 1: Beginner

Builds a capability table over a caller-provided slot backing store and initializes every slot to empty. Give it a slice of slots; it returns either a working table or a typed error.

## Tier 2: Silicon Reality

Construction is O(1) validation (nonzero backing) followed by one O(capacity) initialization pass overwriting every slot with the EMPTY value. The pass is a lower bound: caller-supplied contents are unconstrained, so a deterministic fresh state requires writing every slot once. Failed validation writes nothing.

## Silent Failure Checklist

1. Fallbacks: none; construction either succeeds or returns an error with no partial state.
2. Truncation: none.
3. Wrapping: none; no arithmetic beyond the length check.
4. Error codes: InvalidArgument for a zero-length backing store; classification pinned by test_table_rejects_empty_backing.
