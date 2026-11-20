# FrameAllocator::reserve_range

Function ID: MEM-a03 (domain MEM)

## Tier 1: Beginner

Marks a range of frames as used without handing them out, for kernel-reserved regions. If any frame in the range is already allocated, the whole reservation is rejected and nothing changes.

## Tier 2: Silicon Reality

Two passes over the range: O(len/64) conflict scan (find_one for any set bit) then O(len/64) run set. The conflict pass runs entirely before the first write, so a rejected reservation cannot leave a half-marked region. A complete reserve must inspect every word of the range before writing any of it, so the two-pass structure matches the lower bound for an atomic reserve.

## Silent Failure Checklist

1. Fallbacks: none; conflict rejection is the specified behavior, not a degraded path.
2. Truncation: none; range endpoints are u64 frame numbers compared against the validated window top.
3. Wrapping: end_index - start_index is safe because containment is checked first; allocated += len stays within frame_count by the same containment.
4. Error codes: InvalidArgument for empty or out-of-window ranges; RegionInUse when any overlapped bit is set, verified atomic by the reserve harness.
