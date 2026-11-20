// CEP:FILE: security/threat_model.md
// CEP:WHAT: Initial threat model registry entry for the CIV kernel foundation.
// CEP:WHY: design.md 9.6 requires every security property to identify attacker, capability, target, and residual risk before implementation.
// CEP:CLASS: CC
// CEP:STATUS: partial
// CEP:OWNER: civ-maintainer
// CEP:TICKET: CIV-1
// CEP:FAILURE: none; document only.
// CEP:ASSUMES: none; document only.
// CEP:COST: cold; document maintenance only.
// CEP:EVIDENCE: review by 10-agent vote, verification/evidence/.
// CEP:SECURITY: This document is normative for all kernel foundation modules.

# CIV Threat Model Registry

Per design.md section 9.6, every security property identifies: attacker, capability,
target, attack surface, required protection, assumptions, verification technique,
and residual risk. Entries are versioned; a change to any field is a registry update.

## TM-1: Malicious or buggy device input

| Field | Value |
|---|---|
| Attacker | Device with DMA or interrupt path, or hostile peripheral firmware |
| Capability | Inject arbitrary byte sequences into structures the kernel parses |
| Target | Physical memory allocator metadata; capability table slots |
| Attack surface | Frame allocator `free` path; capability index decoding |
| Required protection | Reject indices outside the initialized frame range; reject slot indices outside the table |
| Assumptions | The allocator bitmap backing store is kernel-owned memory |
| Verification technique | Kani bounded model checking of index arithmetic; adversarial unit tests per standards 61.4 |
| Residual risk | Attacks outside the modeled input bounds are excluded from proofs; documented per proof |

## TM-2: Malicious caller with invalid arguments

| Field | Value |
|---|---|
| Attacker | Kernel-internal caller passing out-of-range or stale handles |
| Capability | Call allocator or capability-table APIs with arbitrary arguments |
| Target | Frame double-free; capability escalation by re-deriving unattenuated rights |
| Attack surface | All public API entry points of `civ-pmm` and `civ-capability` |
| Required protection | Total functions returning typed errors; no panic on any input; attenuation-only derivation |
| Assumptions | None; inputs are treated as untrusted |
| Verification technique | Kani proof of panic freedom over symbolic arguments (free target: full u64 range; frame counts bounded per harness with unit tests covering the bound edges); adversarial unit tests per standards 61.4 |
| Residual risk | None within the modeled ranges; the unbounded frame-count range is the declared FV-1 refinement gap (unit test test_new_rejects_overflow_base covers the window-top edge) |

## TM-3: Capability forgery

| Field | Value |
|---|---|
| Attacker | Code holding an arbitrary 64-bit value attempting to use it as a capability handle |
| Capability | Present any bit pattern to `CapabilityTable::lookup` |
| Target | Capability integrity (design.md 21: Capability integrity) |
| Attack surface | `lookup`, `derive`, `revoke` |
| Required protection | Handles encode slot index plus generation; stale generation fails closed; derived rights are a subset of parent rights |
| Assumptions | The generation counter is not forgeable within the proof model (monotone per slot) |
| Verification technique | Kani: attenuation monotonicity, revocation-closes-access, generation monotonicity |
| Residual risk | Handle guessing across process boundaries is excluded until the process manager exists (design.md 3.2); tracked as CIV-3 |

## Residual-risk registry

| ID | Description | Ticket |
|---|---|---|
| CIV-2 | Capability handle entropy against cross-process guessing requires the process manager | CIV-3 |
| CIV-3 | Per-commit Hunt Receipt infrastructure (standards 65) requires a CI runner with a signing key | CEP-90 |
| CIV-4 | PMC-backed benchmark receipts (standards 69) require isolated, frequency-locked hardware | CEP-91 |
