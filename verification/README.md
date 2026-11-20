// CEP:FILE: verification/README.md
// CEP:WHAT: Verification architecture and proof reproduction instructions for the CIV kernel foundation.
// CEP:WHY: CIV verification chain.md requires each proof to carry tool, version, assumptions, result, scope, hash, and reproduction instructions; this document defines how those artifacts are produced.
// CEP:CLASS: CC
// CEP:STATUS: partial
// CEP:OWNER: civ-maintainer
// CEP:TICKET: CIV-1
// CEP:FAILURE: none; document only.
// CEP:ASSUMES: none; document only.
// CEP:COST: cold; document maintenance only.
// CEP:EVIDENCE: tool versions recorded in verification/evidence/toolchain.json.
// CEP:SECURITY: Proof evidence is integrity-critical; falsification is an integrity breach (standards 65.7).

# CIV Verification Architecture

The verification tree follows CIV verification chain.md section 5. This repository
currently implements the first layers:

| Layer | Status | Artifacts |
|---|---|---|
| Requirements and threat model | Started | security/threat_model.md, design.md CIV-REQ-001..015 |
| Kernel implementation proofs | Started | `#[cfg(kani)]` harnesses in each kernel crate; registry in verification/proof-registry.json |
| Compiler, linker, binary verification | UNDEFINED | Not started; requires the kernel build target (design.md 2.2) |
| Boot and deployed-system validation | UNDEFINED | Not started; requires hardware bring-up |

UNDEFINED layers are declared per design.md 0.3; they are not silently assumed complete. Ticket CIV-1 tracks the population of this verification tree itself and is registered in security/threat_model.md alongside the residual-risk entries.

## Proof classes

Per standards 54.2, artifacts declare one of:

- FV-0: machine-checked, complete proofs, zero sorries. Not yet claimed by any artifact.
- FV-1: verified models and bounded proofs with documented gaps (refinement notes mandatory).
- FV-2: proof tooling.

Every Kani harness is FV-1: bounded model checking over symbolic inputs with an
explicit bound documented in the harness and in the proof registry entry. A Kani
result proves the property for all inputs within the declared bound and the
declared model; it does not establish universal real-world correctness. Each
registry entry records this limitation in its scope field (bounded model per
the entry's assumptions); the source statement is CIV verification chain.md
section 2, critical limitation.

## Reproduction

1. Install Kani: `cargo install --locked kani-verifier && cargo kani setup`.
2. Run proofs: `cargo kani --workspace` (harness list per crate in proof-registry.json).
3. Run tests: `cargo test --workspace`.
4. Run clippy: `cargo clippy --workspace --all-targets -- -D warnings`.
5. Record tool versions into verification/evidence/toolchain.json before regenerating
   evidence; evidence without a tool version is invalid (proof registry contract).

## Evidence integrity

Evidence files under verification/evidence/ record command output verbatim plus a
SHA-256 of the source revision. Falsifying evidence is an integrity breach under
standards 65.7. An unverified claim must be labeled UNVERIFIED, never passed off as
a proof.
