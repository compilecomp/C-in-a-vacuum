// CEP:FILE: verification/evidence/votes/civ-capability-round1.md
// CEP:WHAT: Round-1 record of the mandatory 10-agent optimality vote over every civ-capability definition, with the consolidated defect matrix and per-defect disposition.
// CEP:WHY: The project mandate requires 10 agents to vote on the optimality of every definition against the standard; vote verdicts and the fixes they caused are verification evidence (standards 65.7).
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: none; record only.
// CEP:ASSUMES: none; record only.
// CEP:COST: cold; documentation maintenance only.
// CEP:EVIDENCE: the ten voter reports; the fixed tree is this commit.
// CEP:SECURITY: record contains no secrets.

# civ-capability 10-Agent Vote, Round 1

| Voter | Lens | Tally | Verdict |
|---|---|---|---|
| 1 | Optimality (Ch. 23/63) | 4/2 | REJECT |
| 2 | Comment contract (Ch. 10/35) | 57/8 | REJECT |
| 3 | Rust policy (Ch. 25, Laws 1/6) | 59/5 | REJECT |
| 4 | Security (Ch. 22, TM-3) | 60/5 | ACCEPT |
| 5 | Correctness hand-traces | 60/4 | ACCEPT |
| 6 | Testing (Ch. 61/65) | 61/4 | ACCEPT |
| 7 | Formal verification (Ch. 54) | 60/7 | REJECT |
| 8 | Topology/naming (Ch. 32/33/62) | 74/1 | ACCEPT |
| 9 | Rosetta docs (Ch. 68) | 73/5 | REJECT |
| 10 | Adversarial sweep | 57/7 | REJECT |

Round-1 result: 3/10 ACCEPT. Every severity-0/1 defect repaired in this commit.

## Consolidated defect classes and disposition

1. Harness oracle bug (voter 7, highest severity): proof_derive_never_escalates
   computed grant_ok from the symbolic flag while the symbolic parent rights
   could already contain the grant bit, so a correct implementation would
   produce spurious counterexamples. Disposition: oracle now derives grant_ok
   from the parent's actual rights; the subset witness is independent
   raw arithmetic (child & !parent == 0), removing the contains_all oracle
   circularity flagged by voter 10.
2. TM-3 wrap residual (voters 2, 3, 4, 5, 6, 7, 10): four sites cited a
   generation-wrap residual "recorded in threat model TM-3" that TM-3 did
   not contain, and TM-3's "monotone per slot" wording contradicted
   wrapping_add. Disposition: TM-3 now records the 2^32 wrap residual, the
   wrapping semantics, the cascading-revocation gap, and the 22.9 timing
   policy; CIV-5 (cascading revocation) registered in the residual-risk
   registry.
3. Unchecked narrowing (voters 3, 4, 10): alloc/derive narrowed slot
   indexes as u32 without a capacity bound. Disposition: new() now rejects
   backing stores longer than u32::MAX slots (InvalidArgument), the
   table-instance lifetime contract (generation reset across instances) is
   documented, and CS-a02's circular "fits by capacity bound" wording was
   replaced with the enforced bound.
4. False optimality claim (voters 1, 4, 10): derive claimed lower-bound
   optimal while omitting the O(capacity) free-slot scan that alloc in the
   same file declares not-optimal. Disposition: derive relabeled
   not-optimal with the disclosed gap and shared ticket CEP-13; revoke's
   OPTPROOF corrected to the four writes the contract demands; the file
   header COST no longer groups O(1) revoke into the O(capacity) set.
5. Stale test metadata (voters 3, 4, 6, 7, 8, 9): header claimed 28
   scenarios (17/10/1) against 26 actual tests; the recomputed distribution
   (14/11/1) keeps happy at 4 percent under the 61.4.1 cap after
   reclassifying the content-checking allocation test as middle.
6. Arithmetic/wording defects in docs (voters 1, 9): CS-a13 "4096"
   corrected to 2048 (64 parents x 32 requests) in both source comment and
   doc; CS-a14 "63 mutated handles" corrected to 64 with the lossless
   u32 half-extraction truncation answer justified; CS-a03 garbled sweep
   notation corrected to the actual 2^6 x 2^5 shape; CS-a13 object-
   inheritance claim re-attributed to the harness that actually proves it.
7. Dead code and pointer mismatches (voters 2, 3): SlotEntry::rights and
   is_allocated had zero call sites repo-wide (34.4.4) and were
   exterminated; empty/rights_of/capacity evidence pointers retargeted to
   artifacts that actually exercise them; alloc's rationale moved from
   OPTPROOF to OPTNOTE per 23.5.
8. Type-level CEP blocks (voter 1): Rights, CapabilityError, SlotEntry,
   Handle, and CapabilityTable gained full type-level comment contracts
   per 10.2.
9. Design.md 0.3 completeness (voter 10): the crate docs now declare
   capability transfer between processes and the authority audit as
   UNDEFINED at this increment alongside the cascading-revocation subset;
   lib.rs invariant 3 qualifies the revocation guarantee with the wrap
   residual; "single verified unit" reworded to proof-backed with
   UNVERIFIED status; the 22.9 side-channel policy statement added to
   lookup's SECURITY field.
10. Registry 14.3 fields (voter 7): all 13 proof entries now carry
    dependencies and artifact_hash fields (explicitly marked UNVERIFIED
    pending recorded runs and CI association).

Recorded observation (not charged): the lib.rs doc phrase "is a declared
future increment" replaced an "is not implemented" phrasing that would
have tripped the repository's own PLACEHOLDER-COMMENT lint suppression
boundary on inner doc comments.
