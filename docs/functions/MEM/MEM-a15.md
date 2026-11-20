# test_boundary_first_and_last_frame

Function ID: MEM-a15 (domain MEM)

## Tier 1: Beginner

Unit test: exercises the first frame, the window top frame, and the exact exhaustion boundary of a 129-frame window spanning three words.

## Tier 2: Silicon Reality

129 is the smallest frame count that is non-multiple-of-64 and spans three words, so the third word is only 1/64 used and the last managed bit (128) is exactly position 0 of word 2, the boundary where an inclusive window-top check would escape the word. The test pins the first allocation to the base frame, walks to exact exhaustion, frees the window-top frame, and asserts the exact same frame is returned, which catches the classic last-word off-by-one class.

## Silent Failure Checklist

1. Fallbacks: n/a; test artifact.
2. Truncation: n/a.
3. Wrapping: n/a inside the test window.
4. Error codes: n/a; test artifact.
