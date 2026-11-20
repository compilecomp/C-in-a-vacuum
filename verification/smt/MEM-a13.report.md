CEP-SMT VERIFICATION REPORT

Target: civ-pmm::reserve_conflict_is_atomic (harness proof_reserve_conflict_is_atomic)
Property: P-MM-RESERVE: reserve returns RegionInUse iff the range intersects the allocation; rejection preserves state; acceptance leaves no free frame inside the range with exhaustion only after the complement drains
Profile: OWNERSHIP + TRANSACTIONALITY

Solver: Kani bounded model checking (UNVERIFIED: awaiting recorded run)
Solver Version: recorded at proof time in verification/evidence/toolchain.json
Translation Engine: Kani GOTOC translation over the shipped crate source (no abstract model)

Maximum Steps: 50
Maximum Path Conditions: 50
Maximum Loop Iterations: 10
Solver Timeout: 30 seconds
Memory Limit: 512 MiB

Result: UNVERIFIED (expected classification on a recorded run: UNSAT per CEP-SMT-01 2.3; UNKNOWN/TIMEOUT/BOUND_EXCEEDED will be reported as failure, never as success)
SAT / UNSAT / UNKNOWN / TIMEOUT /
BOUND_EXCEEDED / TRANSLATION_FAILURE

Explored Paths: recorded at proof time by cargo kani
Pruned Paths: input-domain restriction per the assumptions below; pruning preserves all executions relevant to the property
Unexplored Paths: Out-of-window ranges covered by test_reserve_range_out_of_window_rejected

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 64-frame window; range inside the managed window (out-of-window arm covered by unit test)
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Out-of-window ranges covered by test_reserve_range_out_of_window_rejected

Reproduction Command: cargo kani -p civ-pmm --harness proof_reserve_conflict_is_atomic

