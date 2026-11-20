# FrameAllocator::new

Function ID: MEM-a01 (domain MEM)

## Tier 1: Beginner

Builds a frame allocator over an aligned base address and a caller-provided bitmap backing store, and hands it back empty. Give it a base address, a frame count, and a slice of u64 words; it either returns a working allocator or a typed error.

## Tier 2: Silicon Reality

Construction performs O(1) validation (alignment predicate, nonzero count, backing capacity via div_ceil, window-top overflow checks in frame-number and byte-address space) followed by one O(words) clearing pass over the backing store. The clear is a lower bound, not overhead: caller-supplied backing contents are unconstrained, so any deterministic fresh state requires writing every word once. Failed validation writes nothing; clear_all runs only after all checks pass.

## Silent Failure Checklist

1. Fallbacks: none; construction either succeeds or returns an error with no partial state.
2. Truncation: backing.len() and required word count are compared as u64 after widening; usize::try_from rejects counts beyond usize range with Overflow instead of truncating.
3. Wrapping: base_frame + frame_count and window_top * FRAME_SIZE_BYTES are both checked; Overflow is returned rather than wrapped.
4. Error codes: InvalidArgument, BitmapTooSmall, Overflow; each classification is pinned by a unit test and the Kani construction harness.
