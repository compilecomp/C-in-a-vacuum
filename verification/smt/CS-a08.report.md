CEP-SMT VERIFICATION REPORT

Target: civ-capability::derive_never_escalates (harness proof_derive_never_escalates)
Property: P-CS-ESCALATION: derive returns Ok iff (parent grant right AND request subset of parent rights); child rights equal the request on the parent object; every refusal classifies exactly
Profile: OWNERSHIP

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
Unexplored Paths: 4-slot capacity; cascading revocation unimplemented (CIV-5)

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 4-slot table; parent allocated first; rights symbolic over full u64; symbolic grant flag
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. 4-slot capacity; cascading revocation unimplemented (CIV-5)

Reproduction Command: cargo kani -p civ-capability --harness proof_derive_never_escalates

