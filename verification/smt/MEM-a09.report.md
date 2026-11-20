CEP-SMT VERIFICATION REPORT

Target: civ-pmm::new_rejects_bad_config_no_panic (harness proof_new_rejects_bad_config_no_panic)
Property: P-MM-CONSTRUCT: new() returns Ok iff (base frame-aligned AND count > 0 AND backing words sufficient AND window addressable); no panic in the bounded space
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
Unexplored Paths: Unbounded frame counts and the unrepresentable-window-top arm are outside the model (unit test test_new_rejects_overflow_base covers the latter)

Counterexample: none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)
Minimized Counterexample: n/a until a SAT result exists none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)

Assumptions: count <= 128; backing <= 4 words; address <= u64::MAX - FRAME_SIZE_BYTES*128; FRAME_SIZE_BYTES = 4096
Limitations: Evidence class: E2 (expected): all modeled paths within bounds satisfy the property. Unbounded frame counts and the unrepresentable-window-top arm are outside the model (unit test test_new_rejects_overflow_base covers the latter)

Reproduction Command: cargo kani -p civ-pmm --harness proof_new_rejects_bad_config_no_panic

