// CEP:FILE: source/kernel/civ-pmm/src/bitmap.rs
// CEP:WHAT: A fixed-capacity allocation bitmap over a caller-provided word slice, with word-parallel run search and run set operations.
// CEP:WHY: The allocator's correctness reduces to bitmap correctness; isolating the bit array behind a bounds-checked, panic-free API (standards 62.1.1 one data structure per file) gives Kani one narrow surface to prove and keeps the free-frame scan word-parallel for the design.md 4.1 page allocator.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: All fallible operations return PmmError (InvalidArgument on out-of-bounds or zero-length runs); no panic on any input within the modeled bounds; no allocation; no I/O.
// CEP:ASSUMES: Bit positions use LSB-first ordering within each word; capacity arithmetic uses saturating multiplication so absurd slice lengths cannot panic (the saturating value is never accepted as a valid index because it fails the limit checks).
// CEP:COST: Single-bit ops O(1); find operations O(words) with word-parallel skipping; run verification O(run/64) words; all allocation-free.
// CEP:EVIDENCE: unit tests bitmap_* in source/kernel/civ-pmm/tests/allocator_tests.rs; Kani proofs proof_bitmap_run_search_consistent, proof_bitmap_set_run_bounds.
// CEP:SECURITY: Inputs are untrusted (threat model TM-1, TM-2); every index is bounds-checked before use and run preconditions are enforced by debug assertions that Kani and test builds exercise (standards Law 3).

//! Allocation bitmap over a caller-provided backing store.

use crate::error::PmmError;

/// Bits per bitmap word (u64).
// CEP:WHAT: Word width of the bitmap in bits.
// CEP:WHY: Named configuration constant instead of a scattered 64 literal (standards Law 7); tied to the u64 element type of the backing slice.
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: Backing words are u64; enforced by the Bitmap type signature.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: crate type definitions in this file.
pub const BITS_PER_WORD: u64 = 64;

/// A borrow-owned allocation bitmap: bit `i` in word `i / 64`, LSB-first.
///
/// The bitmap owns no memory; it borrows a mutable word slice. Capacity is
/// `slice.len() * 64` bits, computed with saturating arithmetic so that no
/// slice length can panic during capacity computation.
#[derive(Debug)]
pub struct Bitmap<'a> {
    words: &'a mut [u64],
}

impl<'a> Bitmap<'a> {
    /// Wraps a word slice as a bitmap; initial contents are unconstrained
    /// until `clear_all` or explicit initialization.
    // CEP:WHAT: Wraps a mutable word slice into a bitmap handle.
    // CEP:WHY: Caller-provided backing stores (standards 25.5) keep the allocator allocation-free; the borrow enforces single ownership at compile time.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: Caller initializes contents before first allocation use; the frame allocator calls clear_all during construction, and that sequence is exercised by tests and proofs.
    // CEP:COST: O(1); one pointer plus length bind, no copying.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_alloc_free_roundtrip.
    #[inline]
    pub const fn new(words: &'a mut [u64]) -> Self {
        Self { words }
    }

    /// Total bits the backing store can index; saturates at u64::MAX.
    // CEP:WHAT: Returns the bitmap capacity in bits.
    // CEP:WHY: All bounds checks compare against one capacity source; saturating multiplication keeps the computation panic-free for any slice length (standards 66.4).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function; saturation is unreachable for any slice that exists in addressable memory and is rejected by limit checks anyway.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one multiply and one compare.
    // CEP:EVIDENCE: Kani proof proof_bitmap_set_run_bounds over symbolic lengths.
    #[inline]
    pub const fn bits_capacity(&self) -> u64 {
        (self.words.len() as u64).saturating_mul(BITS_PER_WORD)
    }

    /// Words required to index `bits` bits; overflow-safe conversion to usize.
    // CEP:WHAT: Computes the backing-store word count required for a requested bit count.
    // CEP:WHY: Construction-time validation (BitmapTooSmall classification) needs the exact requirement; div_ceil avoids the classic off-by-one of hand-rolled rounding.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::Overflow when the required word count exceeds the usize range of the target.
    // CEP:ASSUMES: none; input may be any u64 (threat model TM-2).
    // CEP:COST: O(1); one divide plus one conversion check.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_new_rejects_undersized_bitmap; Kani proof proof_new_rejects_bad_config_no_panic.
    #[inline]
    pub fn required_words(bits: u64) -> Result<usize, PmmError> {
        let words = bits.div_ceil(BITS_PER_WORD);
        usize::try_from(words).map_err(|_| PmmError::Overflow)
    }

    /// Zeroes every word; used once at construction.
    // CEP:WHAT: Clears all allocation bits to zero.
    // CEP:WHY: The allocator's "fresh" state must be independent of caller-provided memory contents; uninitialized backing stores must never surface as phantom allocations (design.md 4.5 no uninitialized disclosure).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function over any backing store.
    // CEP:ASSUMES: none.
    // CEP:COST: O(words); single sequential store pass, memset-lowering eligible; no allocation.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_new_rejects_undersized_bitmap (fresh state asserted); Kani proof proof_invariant_popcount_matches_allocated.
    #[inline]
    pub fn clear_all(&mut self) {
        for word in self.words.iter_mut() {
            *word = 0;
        }
    }

    /// Reads allocation bit `bit`.
    // CEP:WHAT: Returns the value of one allocation bit with bounds checking.
    // CEP:WHY: Free-path validation (double-free rejection) reads before clearing; a checked read keeps that path total (threat model TM-2).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when bit >= capacity.
    // CEP:ASSUMES: none; any bit index accepted.
    // CEP:COST: O(1); one bounds check, one load, one shift-mask.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_bitmap_out_of_bounds_rejected.
    #[inline]
    pub fn get(&self, bit: u64) -> Result<bool, PmmError> {
        let capacity = self.bits_capacity();
        if bit >= capacity {
            return Err(PmmError::InvalidArgument);
        }
        let word = self.words[(bit / BITS_PER_WORD) as usize];
        Ok(word & (1u64 << (bit % BITS_PER_WORD)) != 0)
    }

    /// Sets allocation bit `bit` to one.
    // CEP:WHAT: Marks one bit allocated with bounds checking.
    // CEP:WHY: Single-frame allocation needs an O(1) set; bounds checks keep adversarial indices total (threat model TM-2).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when bit >= capacity.
    // CEP:ASSUMES: none; run-level invariants (set only over verified-zero runs) are enforced one layer up by the allocator and by debug assertions in set_run.
    // CEP:COST: O(1); one bounds check, one load, one or, one store.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_bitmap_out_of_bounds_rejected.
    #[inline]
    pub fn set(&mut self, bit: u64) -> Result<(), PmmError> {
        let capacity = self.bits_capacity();
        if bit >= capacity {
            return Err(PmmError::InvalidArgument);
        }
        self.words[(bit / BITS_PER_WORD) as usize] |= 1u64 << (bit % BITS_PER_WORD);
        Ok(())
    }

    /// Clears allocation bit `bit` to zero.
    // CEP:WHAT: Marks one bit free with bounds checking.
    // CEP:WHY: The free path clears exactly one verified bit; keeping the primitive total lets the allocator prove state-unchanged-on-error.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when bit >= capacity.
    // CEP:ASSUMES: Caller verified the bit is currently set (allocator free path checks via get first); the invariant sequence get-then-clear is exercised by tests and Kani proof proof_free_validates_untrusted_input.
    // CEP:COST: O(1); one bounds check, one load, one and-not, one store.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_double_free_rejected; Kani proof proof_free_validates_untrusted_input.
    #[inline]
    pub fn clear(&mut self, bit: u64) -> Result<(), PmmError> {
        let capacity = self.bits_capacity();
        if bit >= capacity {
            return Err(PmmError::InvalidArgument);
        }
        self.words[(bit / BITS_PER_WORD) as usize] &= !(1u64 << (bit % BITS_PER_WORD));
        Ok(())
    }

    /// First zero bit in `[from, limit)`; `None` when the interval holds none.
    // CEP:ID: MEM-a04
    // CEP:WHAT: Finds the lowest free bit at or after `from`, bounded by `limit`.
    // CEP:WHY: Word-parallel skipping (u64::MAX words are rejected in one step) makes the first-fit scan amortized-cheap while staying simple enough to prove; a rotating scan hint in the allocator turns this into O(1) amortized allocation.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when limit exceeds capacity or from exceeds limit.
    // CEP:ASSUMES: none beyond the bounds contract.
    // CEP:COST: worst-case O(limit/64) word loads; full words are skipped in one load; no allocation.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: any correct search over a bit array must at minimum load each word that can contain a candidate before the answer; this implementation loads exactly the words from `from` to the answer, skipping full words in one load, so its load count matches the lower bound for a LSB-first first-free search. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91). Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_bitmap_find_zero_patterns; Kani proof proof_bitmap_run_search_consistent.
    pub fn find_zero(&self, from: u64, limit: u64) -> Result<Option<u64>, PmmError> {
        let capacity = self.bits_capacity();
        if limit > capacity || from > limit {
            return Err(PmmError::InvalidArgument);
        }
        let mut bit = from;
        while bit < limit {
            let word_index = (bit / BITS_PER_WORD) as usize;
            let position = (bit % BITS_PER_WORD) as u32;
            let word = self.words[word_index];
            let masked = word | ((1u64 << position) - 1);
            if masked != u64::MAX {
                let candidate =
                    (word_index as u64) * BITS_PER_WORD + u64::from(masked.trailing_ones());
                if candidate < limit {
                    return Ok(Some(candidate));
                }
                return Ok(None);
            }
            bit = (word_index as u64 + 1) * BITS_PER_WORD;
        }
        Ok(None)
    }

    /// First set bit in `[from, limit)`; `None` when the interval holds none.
    // CEP:ID: MEM-a05
    // CEP:WHAT: Finds the lowest allocated bit at or after `from`, bounded by `limit`.
    // CEP:WHY: Reserve-conflict detection (atomic RegionInUse classification) needs the first conflicting allocation; symmetric to find_zero with the opposite polarity.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when limit exceeds capacity or from exceeds limit.
    // CEP:ASSUMES: none beyond the bounds contract.
    // CEP:COST: worst-case O(limit/64) word loads; zero words skipped in one load; no allocation.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: same load-count lower bound as find_zero with reversed polarity; zero words are skipped in one load. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91). Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_reserve_range_conflict_atomic; Kani proof proof_reserve_conflict_is_atomic.
    pub fn find_one(&self, from: u64, limit: u64) -> Result<Option<u64>, PmmError> {
        let capacity = self.bits_capacity();
        if limit > capacity || from > limit {
            return Err(PmmError::InvalidArgument);
        }
        let mut bit = from;
        while bit < limit {
            let word_index = (bit / BITS_PER_WORD) as usize;
            let position = (bit % BITS_PER_WORD) as u32;
            let word = self.words[word_index];
            let masked = word & !((1u64 << position) - 1);
            if masked != 0 {
                let candidate =
                    (word_index as u64) * BITS_PER_WORD + u64::from(masked.trailing_zeros());
                if candidate < limit {
                    return Ok(Some(candidate));
                }
                return Ok(None);
            }
            bit = (word_index as u64 + 1) * BITS_PER_WORD;
        }
        Ok(None)
    }

    /// True when every bit in `[start, start + run_len)` is zero.
    // CEP:ID: MEM-a06
    // CEP:WHAT: Verifies a run of bits is entirely free, word-parallel.
    // CEP:WHY: set_run's precondition (target run free) must be mechanically enforced, not comment-asserted (standards Law 3); this check is the debug-enforced form and the proof surface for Kani.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument when the run exceeds capacity.
    // CEP:ASSUMES: Caller keeps start + run_len within capacity; enforced here by the checked end computation.
    // CEP:COST: O(run/64 + 1) word loads with masked segment testing; no allocation.
    // CEP:EVIDENCE: Kani proof proof_bitmap_run_search_consistent (agreement between find_zero_run result and this verifier).
    pub(crate) fn run_is_zero(&self, start: u64, run_len: u64) -> Result<bool, PmmError> {
        let capacity = self.bits_capacity();
        let end = start.checked_add(run_len).ok_or(PmmError::Overflow)?;
        if end > capacity {
            return Err(PmmError::InvalidArgument);
        }
        let mut bit = start;
        while bit < end {
            let word_index = (bit / BITS_PER_WORD) as usize;
            let position = (bit % BITS_PER_WORD) as u32;
            let word = self.words[word_index];
            let span_in_word = BITS_PER_WORD - u64::from(position);
            let take = span_in_word.min(end - bit);
            let take_mask = if take >= BITS_PER_WORD {
                u64::MAX
            } else {
                (1u64 << take) - 1
            };
            // Window shift is safe: take <= BITS_PER_WORD - position keeps the
            // highest window bit at or below bit 63.
            let window = take_mask << position;
            if word & window != 0 {
                return Ok(false);
            }
            bit += take;
        }
        Ok(true)
    }

    /// First position where `run_len` consecutive zero bits fit in `[from, limit)`; `None` when none fits.
    // CEP:ID: MEM-a07
    // CEP:WHAT: First-fit contiguous free-run search with word-parallel conflict skipping.
    // CEP:WHY: DMA buffers and page-table pages need contiguous frames (design.md 4.2 huge-page management); first-fit over the rotating hint avoids a full-array pass per request while the conflict-skip keeps adversarial alternation patterns bounded.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument for zero run length, limit above capacity, or from above limit; Ok(None) means the interval cannot host the run (exhaustion is the caller's classification).
    // CEP:ASSUMES: none beyond the bounds contract.
    // CEP:COST: worst-case O(limit/64) word loads plus O(run/64) verification per candidate; each rejected candidate advances the search past its first conflict, so total verification work does not revisit bits; no allocation.
    // CEP:OPTIMAL: not-optimal
    // CEP:OPTPROOF: the search matches the exclusion lower bound (Omega(p/64) word loads plus Omega(run/64) verification words) up to the disclosed constant factor below.
    // CEP:OPTNOTE: the candidate-verification helper re-loads words already skipped as full, so a single fused scan would be cheaper; the shared verifier keeps one proof surface for tests, proofs, and the Law-3 assertion, which 23.8 weights above the word saving. Optimization tracked as ticket CEP-12.
    // CEP:EVIDENCE: cargo test -p civ-pmm test_bitmap_find_zero_patterns, test_alloc_contiguous_after_fragmentation; Kani proof proof_bitmap_run_search_consistent.
    pub fn find_zero_run(
        &self,
        from: u64,
        run_len: u64,
        limit: u64,
    ) -> Result<Option<u64>, PmmError> {
        if run_len == 0 {
            return Err(PmmError::InvalidArgument);
        }
        let capacity = self.bits_capacity();
        if limit > capacity || from > limit {
            return Err(PmmError::InvalidArgument);
        }
        let mut search = from;
        loop {
            if !search
                .checked_add(run_len)
                .is_some_and(|window_end| window_end <= limit)
            {
                return Ok(None);
            }
            let candidate = match self.find_zero(search, limit)? {
                Some(candidate) => candidate,
                None => return Ok(None),
            };
            let probe_end = match candidate.checked_add(run_len) {
                Some(end) if end <= limit => end,
                _ => return Ok(None),
            };
            if self.run_is_zero(candidate, run_len)? {
                return Ok(Some(candidate));
            }
            let conflict = match self.find_one(candidate, probe_end)? {
                Some(position) => position,
                None => return Err(PmmError::InvalidArgument),
            };
            search = conflict + 1;
        }
    }

    /// Sets every bit in `[start, start + run_len)` to one.
    // CEP:ID: MEM-a08
    // CEP:WHAT: Marks a contiguous run allocated with word-parallel writes.
    // CEP:WHY: Contiguous allocation must flip a whole run in O(run/64) word operations instead of per-bit branches; the debug assertion mechanically enforces the free-run precondition instead of trusting callers (standards Law 3).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns PmmError::InvalidArgument for zero run length or a run crossing capacity; debug builds additionally assert the run is free (the allocator guarantees this by construction from find_zero_run).
    // CEP:ASSUMES: The run was verified free immediately before the call (allocator: find_zero_run result; proofs: harness-established state); enforced by debug assertion, not trusted.
    // CEP:COST: O(run/64 + 1) masked word writes; no allocation, no branches per bit.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: flipping n consecutive bits spanning w words requires at least w read-modify-write word operations; the implementation performs exactly one RMW per touched word. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91). Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-pmm test_bitmap_find_zero_patterns; Kani proof proof_bitmap_set_run_bounds.
    pub fn set_run(&mut self, start: u64, run_len: u64) -> Result<(), PmmError> {
        if run_len == 0 {
            return Err(PmmError::InvalidArgument);
        }
        let capacity = self.bits_capacity();
        let end = start.checked_add(run_len).ok_or(PmmError::Overflow)?;
        if start >= capacity || end > capacity {
            return Err(PmmError::InvalidArgument);
        }
        debug_assert!(
            matches!(self.run_is_zero(start, run_len), Ok(true)),
            "set_run precondition violated: run not free"
        );
        let mut bit = start;
        while bit < end {
            let word_index = (bit / BITS_PER_WORD) as usize;
            let position = (bit % BITS_PER_WORD) as u32;
            let span_in_word = BITS_PER_WORD - u64::from(position);
            let take = span_in_word.min(end - bit);
            let take_mask = if take >= BITS_PER_WORD {
                u64::MAX
            } else {
                (1u64 << take) - 1
            };
            self.words[word_index] |= take_mask << position;
            bit += take;
        }
        Ok(())
    }

    /// Number of set bits; the allocator's allocated-frame accounting cross-check.
    // CEP:WHAT: Counts allocation bits set across the backing store.
    // CEP:WHY: Invariant 2 of the crate (allocated_count equals bitmap popcount) needs a mechanical witness for tests, proofs, and runtime telemetry (design.md 1.3 Algorithms: free-page accounting).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(words); one popcount instruction per word, allocation-free.
    // CEP:EVIDENCE: Kani proof proof_invariant_popcount_matches_allocated; cargo test -p civ-pmm test_stats_consistency.
    #[inline]
    pub fn popcount(&self) -> u64 {
        self.words
            .iter()
            .map(|word| u64::from(word.count_ones()))
            .sum()
    }
}
