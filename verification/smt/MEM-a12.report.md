CEP-SMT VERIFICATION REPORT

Target: civ-pmm::contiguous_bounds_and_disjoint (harness proof_contiguous_bounds_and_disjoint)
Property: P-MM-CONTIG: contiguous allocation returns exactly the run inside the window; successive runs never overlap; fresh-allocator failure class is Exhausted-only
Profile: BOUNDS + OWNERSHIP

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
Unexplored Paths: Fragmented multi-word windows covered by unit tests

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: count in 1..=64; run in 1..=count; single-word backing
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Fragmented multi-word windows covered by unit tests

Reproduction Command: cargo kani -p civ-pmm --harness proof_contiguous_bounds_and_disjoint

