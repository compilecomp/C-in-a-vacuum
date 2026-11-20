CEP-SMT VERIFICATION REPORT

Target: civ-pmm::bitmap_run_search_consistent (harness proof_bitmap_run_search_consistent)
Property: P-MM-RUNSEARCH: find_zero_run returns Some(p) iff the run at p is verified free and minimal, None iff no origin hosts the run, Err only on structural violations
Profile: BOUNDS

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
Unexplored Paths: Multi-word windows covered by allocator unit tests

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: Single-word bitmap (64 bits); run in 1..=64; window within the word
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Multi-word windows covered by allocator unit tests

Reproduction Command: cargo kani -p civ-pmm --harness proof_bitmap_run_search_consistent

