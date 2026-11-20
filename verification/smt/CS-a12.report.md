CEP-SMT VERIFICATION REPORT

Target: civ-capability::invariant_slots_consistent (harness proof_invariant_slots_consistent)
Property: P-CS-ACCOUNT: allocated_count == slot popcount <= capacity and allocated + free == capacity after every operation in a nondeterministic sequence
Profile: ACCOUNTING

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
Unexplored Paths: Sequence length bounded at 4 within the model

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 4-slot table; 4 nondeterministic operations
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Sequence length bounded at 4 within the model

Reproduction Command: cargo kani -p civ-capability --harness proof_invariant_slots_consistent

