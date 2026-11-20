#!/usr/bin/env python3
# CEP:FILE: scripts/gen_smt_reports.py
# CEP:WHAT: Generates the CEP-SMT-01 section-8 verification reports for every registered proof harness from a structured content table.
# CEP:WHY: SOLVER-TOOLING.md section 8 requires every search to produce a report stating target, property, profile, solver, bounds, result classification, counterexample status, assumptions, limitations, and reproduction; generating from one table keeps the reports synchronized with verification/proof-registry.json (Law 8).
# CEP:CLASS: CEP-2
# CEP:STATUS: complete
# CEP:FAILURE: Exits non-zero if a report target lacks content or a registry proof ID has no report.
# CEP:ASSUMES: Result fields are recorded only from actual tool runs; pending runs stay UNVERIFIED with the expected classification labeled as such (CEP-SMT-01 2.3: UNKNOWN/TIMEOUT/BOUND_EXCEEDED are never reported as success).
# CEP:COST: offline tool; runtime cost irrelevant.
# CEP:EVIDENCE: reports in verification/smt/ cross-checked against proof-registry.json each run.
# CEP:SECURITY: reads and writes repository-trusted paths only; no network.
"""Deterministic CEP-SMT-01 report generator (CEP-2 tooling)."""
import json
import os
import sys

# proof_id -> (crate, harness, property, profile, budget, evidence_class, assumptions, limitations)
ENTRIES = {
 "MEM-a09": ("civ-pmm", "proof_new_rejects_bad_config_no_panic",
   "P-MM-CONSTRUCT: new() returns Ok iff (base frame-aligned AND count > 0 AND backing words sufficient AND window addressable); no panic in the bounded space",
   "CONTROL_FLOW + BOUNDS",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "count <= 128; backing <= 4 words; address <= u64::MAX - FRAME_SIZE_BYTES*128; FRAME_SIZE_BYTES = 4096",
   "Unbounded frame counts and the unrepresentable-window-top arm are outside the model (unit test test_new_rejects_overflow_base covers the latter)"),
 "MEM-a10": ("civ-pmm", "proof_alloc_exhaustion_and_uniqueness",
   "P-MM-UNIQ: allocation yields distinct frames; exhaustion is exact; free-then-alloc reuses the exact freed frame",
   "OWNERSHIP + ACCOUNTING",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "count in 1..=64; single-word backing; PROOF_BASE frame-aligned",
   "Multi-word windows covered by unit tests; unbounded universality not claimed"),
 "MEM-a11": ("civ-pmm", "proof_free_validates_untrusted_input",
   "P-MM-FREE: free accepts exactly the allocated frame; every other u64 fails closed; rejection preserves counter and popcount; double free rejected",
   "OWNERSHIP",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "64-frame window; free target spans full u64",
   "Single-word backing; popcount cross-check asserted on the double-free path, counter on every rejection path"),
 "MEM-a12": ("civ-pmm", "proof_contiguous_bounds_and_disjoint",
   "P-MM-CONTIG: contiguous allocation returns exactly the run inside the window; successive runs never overlap; fresh-allocator failure class is Exhausted-only",
   "BOUNDS + OWNERSHIP",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "count in 1..=64; run in 1..=count; single-word backing",
   "Fragmented multi-word windows covered by unit tests"),
 "MEM-a13": ("civ-pmm", "proof_reserve_conflict_is_atomic",
   "P-MM-RESERVE: reserve returns RegionInUse iff the range intersects the allocation; rejection preserves state; acceptance leaves no free frame inside the range with exhaustion only after the complement drains",
   "OWNERSHIP + TRANSACTIONALITY",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "64-frame window; range inside the managed window (out-of-window arm covered by unit test)",
   "Out-of-window ranges covered by test_reserve_range_out_of_window_rejected"),
 "MEM-a14": ("civ-pmm", "proof_invariant_popcount_matches_allocated",
   "P-MM-ACCOUNT: allocated_count == bitmap popcount and allocated + free == frame_count after every operation in a nondeterministic sequence",
   "ACCOUNTING",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "64-frame window; 4 nondeterministic operations",
   "Sequence length bounded at 4 within the model"),
 "MEM-a15h": ("civ-pmm", "proof_bitmap_run_search_consistent",
   "P-MM-RUNSEARCH: find_zero_run returns Some(p) iff the run at p is verified free and minimal, None iff no origin hosts the run, Err only on structural violations",
   "BOUNDS",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "Single-word bitmap (64 bits); run in 1..=64; window within the word",
   "Multi-word windows covered by allocator unit tests"),
 "MEM-a16h": ("civ-pmm", "proof_bitmap_set_run_bounds",
   "P-MM-SETRUN: Ok(set_run) over a verified-free run raises popcount by exactly run_len and sets every run bit; structural violations are rejected without mutation",
   "BOUNDS",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "Single-word bitmap; run in 1..=64 within bounds; call guarded by run_is_zero",
   "Multi-word behavior covered by allocator unit tests"),
 "MEM-a19": ("civ-pmm", "proof_operations_are_transactional",
   "P-MM-TRANSACTIONALITY: for every fallible operation R = Err implies S1 == S0 over all bitmap words, the allocation counter, and the scan hint (SOLVER-TOOLING.md section 5)",
   "TRANSACTIONALITY",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "64 frames; single-word backing; three symbolic operations over symbolic arguments",
   "Operation sequence bounded at 3 within the model; scan-hint equality is part of the witness"),
 "CS-a08": ("civ-capability", "proof_derive_never_escalates",
   "P-CS-ESCALATION: derive returns Ok iff (parent grant right AND request subset of parent rights); child rights equal the request on the parent object; every refusal classifies exactly",
   "OWNERSHIP",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; parent allocated first; rights symbolic over full u64; symbolic grant flag",
   "4-slot capacity; cascading revocation unimplemented (CIV-5)"),
 "CS-a09": ("civ-capability", "proof_lookup_fail_closed",
   "P-CS-FORGERY: lookup resolves exactly the live handles with exact contents; every other u64 fails closed as InvalidSlot/SlotEmpty/StaleHandle; no panic in the handle space",
   "CONTROL_FLOW + BOUNDS",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; two live capabilities with symbolic contents; handle over full u64",
   "4-slot capacity within the model"),
 "CS-a10": ("civ-capability", "proof_revoke_closes_access",
   "P-CS-REVOKE: after revoke the handle never resolves; double revoke rejected; slot reuse advances the generation and leaves old handles closed",
   "OWNERSHIP",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; single allocation cycle; generation u32",
   "The 2^32 generation wrap is the declared residual (threat model TM-3)"),
 "CS-a11": ("civ-capability", "proof_handle_space_no_panic",
   "P-CS-TOTALITY: lookup, rights_of, and revoke are total over the full u64 handle space with typed classifications and no panic",
   "CONTROL_FLOW",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; one live capability; handle over full u64",
   "4-slot capacity within the model"),
 "CS-a12": ("civ-capability", "proof_invariant_slots_consistent",
   "P-CS-ACCOUNT: allocated_count == slot popcount <= capacity and allocated + free == capacity after every operation in a nondeterministic sequence",
   "ACCOUNTING",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; 4 nondeterministic operations",
   "Sequence length bounded at 4 within the model"),
 "CS-a15": ("civ-capability", "proof_operations_are_transactional",
   "P-CS-TRANSACTIONALITY: for every fallible operation R = Err implies S1 == S0 over all slot entries and the accounting counter (CEP-SMT-01 transactionality profile)",
   "TRANSACTIONALITY",
   {"max_steps": 50, "max_path_conditions": 50, "max_loop_iterations": 10, "solver_timeout_s": 30, "memory_limit": "512 MiB"},
   "E2 (expected): all modeled paths within bounds satisfy the property",
   "4-slot table; one live capability; three symbolic operations",
   "Operation sequence bounded at 3 within the model"),
}

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    registry_path = os.path.join(root, 'verification', 'proof-registry.json')
    registry = json.load(open(registry_path, encoding='utf-8'))
    reg_ids = {p['proof_id'] for p in registry['proofs']}
    missing = reg_ids - set(ENTRIES)
    if missing:
        print(f"registry proofs without reports: {sorted(missing)}")
        return 1
    out_dir = os.path.join(root, 'verification', 'smt')
    os.makedirs(out_dir, exist_ok=True)
    for proof_id, (crate, harness, prop, profile, budget, eclass, assumptions, limitations) in ENTRIES.items():
        content = f"""CEP-SMT VERIFICATION REPORT

Target:
Property:
Profile:

Solver:
Solver Version:
Translation Engine:

Maximum Steps:
Maximum Path Conditions:
Maximum Loop Iterations:
Solver Timeout:
Memory Limit:

Result:
SAT / UNSAT / UNKNOWN / TIMEOUT /
BOUND_EXCEEDED / TRANSLATION_FAILURE

Explored Paths:
Pruned Paths:
Unexplored Paths:

Counterexample:
Minimized Counterexample:

Assumptions:
Limitations:

Reproduction Command:
"""
        values = {
            "Target:": f"{crate}::{harness.split('proof_')[-1]} (harness {harness})",
            "Property:": prop,
            "Profile:": profile,
            "Solver:": "Kani bounded model checking (UNVERIFIED: awaiting recorded run)",
            "Solver Version:": "recorded at proof time in verification/evidence/toolchain.json",
            "Translation Engine:": "Kani GOTOC translation over the shipped crate source (no abstract model)",
            "Maximum Steps:": str(budget['max_steps']),
            "Maximum Path Conditions:": str(budget['max_path_conditions']),
            "Maximum Loop Iterations:": str(budget['max_loop_iterations']),
            "Solver Timeout:": f"{budget['solver_timeout_s']} seconds",
            "Memory Limit:": budget['memory_limit'],
            "Result:": "UNVERIFIED (expected classification on a recorded run: UNSAT per CEP-SMT-01 2.3; UNKNOWN/TIMEOUT/BOUND_EXCEEDED will be reported as failure, never as success)",
            "Explored Paths:": "recorded at proof time by cargo kani",
            "Pruned Paths:": "input-domain restriction per the assumptions below; pruning preserves all executions relevant to the property",
            "Unexplored Paths:": limitations,
            "Counterexample:": "none recorded (no run has been executed; any SAT result will be retained and minimized per CEP-SMT-01 section 8)",
            "Minimized Counterexample:": "n/a until a SAT result exists",
            "Assumptions:": assumptions,
            "Limitations:": f"Evidence class: {eclass}. {limitations}",
            "Reproduction Command:": f"cargo kani -p {crate} --harness {harness}",
        }
        for line, value in values.items():
            content = content.replace(line, f"{line} {value}")
        with open(os.path.join(out_dir, f"{proof_id}.report.md"), 'w', encoding='utf-8') as handle:
            handle.write(content + "\n")
    # registry: add smt fields
    for entry in registry['proofs']:
        e = ENTRIES[entry['proof_id']]
        entry['smt_profile'] = e[3]
        entry['search_budget'] = e[4]
        entry['evidence_class'] = e[5]
        entry['property_id'] = e[2].split(':')[0]
    json.dump(registry, open(registry_path, 'w', encoding='utf-8'), indent=2)
    open(registry_path, 'a').write('\n')
    print(f"reports: {len(ENTRIES)} written to verification/smt/; registry updated with profile/budget/class/property fields")
    return 0

if __name__ == '__main__':
    sys.exit(main())
