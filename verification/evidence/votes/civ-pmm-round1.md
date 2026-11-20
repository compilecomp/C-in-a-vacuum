// CEP:FILE: verification/evidence/votes/civ-pmm-round1.md
// CEP:WHAT: Round-1 record of the mandatory 10-agent optimality vote over every civ-pmm definition, with the consolidated defect matrix and per-defect disposition.
// CEP:WHY: The project mandate requires 10 agents to vote on the optimality of every definition against the standard; the vote, its verdicts, and the fixes it caused are verification evidence and must be recorded (standards 65.7 integrity, design.md 14.3 proof registry discipline).
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: none; record only.
// CEP:ASSUMES: none; record only.
// CEP:COST: cold; documentation maintenance only.
// CEP:EVIDENCE: the vote transcript fragments are the 10 agent reports; the fixed tree is this commit.
// CEP:SECURITY: record contains no secrets.

# civ-pmm 10-Agent Vote, Round 1

Voter lenses: 1 optimality (Ch. 23/63), 2 comment contract (Ch. 10/35), 3 Rust
policy (Ch. 25, Laws 1/6), 4 security (Ch. 22, design 4.5, threat model),
5 algorithmic correctness (hand-traces), 6 testing (Ch. 61/65), 7 formal
verification (Ch. 54, CIV verification chain.md), 8 topology/naming
(Ch. 32/33/62), 9 Rosetta docs (Ch. 68, doc standards.md), 10 adversarial
full-standard sweep. Every voter read standards.md in full (15,973 lines).

## Verdicts

| Voter | Lens | Tally (optimal/defect) | Verdict |
|---|---|---|---|
| 1 | Optimality | 37/15 | REJECT |
| 2 | Comment contract | 31/23 | REJECT |
| 3 | Rust policy | 41/6 | REJECT |
| 4 | Security | 45/2 | REJECT |
| 5 | Correctness hand-traces | 37/10 | REJECT |
| 6 | Testing | 61/17 | REJECT |
| 7 | Formal verification | 9/7 | REJECT |
| 8 | Topology/naming | 84/10 | REJECT |
| 9 | Rosetta docs | 78/24 | REJECT |
| 10 | Adversarial sweep | 45/15 | REJECT |

Round-1 result: 0/10 ACCEPT. Per the vote protocol, every severity-0/1 defect
must be repaired and the repair re-reviewed before the module merges.

## Consolidated defect classes and disposition

1. Phantom evidence citations (voters 1, 2, 5, 6, 7, 9): CEP fields cited two
   Kani harnesses (`proof_bitmap_run_search_consistent`,
   `proof_bitmap_set_run_bounds`) and two test names that did not exist.
   Disposition: both harnesses were WRITTEN and added to proofs.rs
   (MEM-a15h, MEM-a16h) with full CEP blocks; the two test-name pointers were
   corrected to the real tests. A mechanical grep now finds zero references
   to nonexistent artifacts.
2. Empty evidence artifacts (all voters): `verification/proof-registry.json`
   was empty and `verification/evidence/` held nothing while being cited.
   Disposition: the registry now carries eight proof entries with the
   design.md 14.3 fields; evidence files (test-run.log, clippy-run.log,
   toolchain.json, this record) exist. Kani entries are labeled UNVERIFIED
   until a recorded `cargo kani` run exists; nothing claims a pass that did
   not happen (standards 65.7).
3. Broken harnesses (voters 3, 4, 5, 7): proof_new omitted the address-top
   overflow arm while sampling the full u64 address range;
   proof_alloc_exhaustion_and_uniqueness lacked an unwind declaration;
   proof_reserve_conflict_is_atomic sampled below-window starts and asserted
   unconditional exhaustion after a partial reserve.
   Disposition: all three repaired (bounded address constraint with the
   refinement gap named; `#[kani::unwind]` declared on every harness after
   round-2 reviewer B verified the initial patch had not fully landed; the
   reserve harness now assumes in-window ranges and proves isolation by
   draining the complement while asserting no reserved frame is returned).
4. Stale claims (voters 1, 3, 6, 8, 9, 10): longest-message 47 -> 56 bytes;
   24 -> 33 scenarios; design.md 1.1 -> 1.3 page-size citation;
   design.md 2.2 -> 1.3 reserved-memory citation; phantom "stats()" and
   "wrap-boundary" debug-assertion references removed; alloc and
   find_zero_run OPTIMAL tags relabeled `not-optimal` with OPTNOTE and
   tickets CEP-11/CEP-12 per 23.9; alloc "hottest path" claim reworded.
5. Optimality/benchmarks (voter 10): after round-2 reviewer C verified the
   initial scope was incomplete, every OPT-1 claim (nine sites) now carries
   the explicit note that the benchmark receipt is pending the CIV-4
   hardware contract (ticket CEP-91), so no performance claim is presented
   as measured when it is not.
6. Coverage gaps (voters 2, 6): FrameRange::contains was dead public API and
   the bitmap get/set/clear out-of-capacity branches had no direct test.
   Disposition: two new tests added (MEM-a17, MEM-a18); suite is 33 tests.
7. Rosetta doc defects (voter 9): MEM-a15 bit-position fact corrected
   (bit 128 is position 0 of word 2, not mid-word); MEM-a11 assertion-scope
   wording made precise.
8. Naming (voter 8): the unprefixed test was renamed
   test_error_display_stable_and_nonempty.
9. Threat-model scoping (voter 7): TM-2 residual-risk wording now states
   which argument ranges are modeled where.
10. Standards-internal conflict noted, not charged to the code (voters 1, 5,
    8, 10): CEP:CLASS: FV-1 on proofs.rs follows chapter 54.7.2's own
    example while chapter 50.3's class enumeration omits FV classes. The
    repository's lint configuration admits FV-0/1/2 on that basis; the
    conflict is recorded here for the standard's maintainers.

## Accepted deviations (recorded, not silently assumed)

- Root documents `doc standards.md` and `CIV verification chain.md` carry
  spaces and capitals, which chapter 33.2 would ban for first-party source
  files. They are the user-authored normative documents referenced by name
  throughout the standard; renaming them is a maintainer decision and is
  not taken unilaterally.
- Chapter 50's `cep_lint` tool and the per-commit Bloodhound receipt chain
  are planned increments (ticket CEP-90, registered as CIV-3 in the
  security/threat_model.md residual-risk registry); this round's review
  was performed by the 10 agents directly against the standard text.

## Round 2: delta review of the repairs

Three fresh reviewers verified the repaired tree against the full standard
(full standards.md read in each case):

| Reviewer | Lens | Verdict | Severity-0/1 findings |
|---|---|---|---|
| A | Evidence integrity (citation resolution, registry/doc agreement) | FAIL (3) | CIV-1 ticket unresolvable; header unwind claim 6/8; "quoted verbatim" claim false |
| B | Harness semantics (hand-trace vs Kani model) | FAIL (3) | MEM-a13 patch had not landed; MEM-a15h/a16h blocked by private run_is_zero; MEM-a16h assume overflow and debug-assert trap |
| C | Comment contract and optimality claims | FAIL (3) | MEM-a13 doc/code mismatch; kani "results recorded" wording; benchmark-note ledger claim overbroad |

Disposition of every round-2 finding (all repaired in this commit):

1. MEM-a13: in-window assume added, complement-drain loop added with
   no-reserved-frame and post-drain-exhaustion assertions, unwind(72)
   declared; registry, FV-REFINEMENT, docstring, and MEM-a13.md aligned
   with the implemented bound.
2. run_is_zero made pub(crate) so the proof crate compiles under cfg(kani).
3. MEM-a16h: overflow-safe assumes (start <= 64 - run_len), guarded call
   (set_run invoked only over verified-free runs; structural rejections
   covered by concrete Err calls), header and doc wording aligned.
4. MEM-a11: unwind(4) declared; header claim now true for all 8 harnesses.
5. Kani EVIDENCE wording changed to "UNVERIFIED until a recorded run"
   across the file header and all harness blocks.
6. verification/README.md no longer claims a verbatim registry quote;
   the scope-field caveat wording is used instead.
7. CIV-1 registered in the threat-model residual-risk registry.
8. Benchmark-pending note added to all nine OPT-1 OPTPROOF sites; the
   round-1 ledger item corrected to describe the actual scope.
9. Duplicate CEP:ID lines removed; duplicate OPTNOTE removed; design.md
   1.3 Algorithms cited for free-page accounting; exact test names cited.

Residual observation (recorded, not repaired): reviewer B notes the
MEM-a16h Err arm is vacuous inside the assumed bounds by construction;
the concrete structural-Err calls keep the classification pinned.
