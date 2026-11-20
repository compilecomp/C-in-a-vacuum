CEP-SMT VERIFICATION REPORT

Target: civ-pmm::operations_are_transactional (harness proof_operations_are_transactional)
Property: P-MM-TRANSACTIONALITY: for every fallible operation R = Err implies S1 == S0 over all bitmap words, the allocation counter, and the scan hint (SOLVER-TOOLING.md section 5)
Profile: TRANSACTIONALITY

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
Unexplored Paths: Operation sequence bounded at 3 within the model; scan-hint equality is part of the witness

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 64 frames; single-word backing; three symbolic operations over symbolic arguments
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Operation sequence bounded at 3 within the model; scan-hint equality is part of the witness

Reproduction Command: cargo kani -p civ-pmm --harness proof_operations_are_transactional

