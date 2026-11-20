# test_frame_range_contains

Function ID: MEM-a18 (domain MEM)

## Tier 1: Beginner

Unit test: FrameRange::contains classification at both boundaries and for an empty range.

## Tier 2: Silicon Reality

contains is the ownership predicate future page-table consumers rely on (design.md 4.2); this test pins inclusive-start, exclusive-end behavior at both edges and the empty-range answer, so the predicate cannot drift while it awaits its first kernel caller.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: n/a.
3. Wrapping: n/a.
4. Error codes: n/a; test artifact.
