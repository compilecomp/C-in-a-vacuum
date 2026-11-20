CEP-SMT VERIFICATION REPORT

Target: civ-capability::operations_are_transactional (harness proof_operations_are_transactional)
Property: P-CS-TRANSACTIONALITY: for every fallible operation R = Err implies S1 == S0 over all slot entries and the accounting counter (CEP-SMT-01 transactionality profile)
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
Unexplored Paths: Operation sequence bounded at 3 within the model

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 4-slot table; one live capability; three symbolic operations
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Operation sequence bounded at 3 within the model

Reproduction Command: cargo kani -p civ-capability --harness proof_operations_are_transactional

