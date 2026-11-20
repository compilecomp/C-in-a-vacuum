CEP-SMT VERIFICATION REPORT

Target: civ-capability::lookup_fail_closed (harness proof_lookup_fail_closed)
Property: P-CS-FORGERY: lookup resolves exactly the live handles with exact contents; every other u64 fails closed as InvalidSlot/SlotEmpty/StaleHandle; no panic in the handle space
Profile: CONTROL_FLOW + BOUNDS

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
Unexplored Paths: 4-slot capacity within the model

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: 4-slot table; two live capabilities with symbolic contents; handle over full u64
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. 4-slot capacity within the model

Reproduction Command: cargo kani -p civ-capability --harness proof_lookup_fail_closed

