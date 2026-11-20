// CEP:FILE: verification/tickets.md
// CEP:WHAT: The repository ticket ledger resolving every CEP-n and CIV-n ticket cited in code comments and evidence records.
// CEP:WHY: Standards 10.10 bans anonymous or unresolvable work markers; a ticket citation is only meaningful if it resolves to a registered row with an owner and a disposition.
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: none; record only.
// CEP:ASSUMES: Tickets cited in code must exist here in the same commit that cites them; the ledger is updated with every change that adds or resolves a ticket.
// CEP:COST: cold; documentation maintenance only.
// CEP:EVIDENCE: mechanical resolution sweep in verification/evidence/; round-2 delta reviews.
// CEP:SECURITY: record contains no secrets.

# CIV Ticket Ledger

| Ticket | Description | Owner | Status | Opened |
|---|---|---|---|---|
| CIV-1 | Verification-tree population: record cargo kani runs and evidence artifacts for all registered proofs | civ-maintainer | open | scaffold increment |
| CIV-2 | Residual-risk bookkeeping entry superseded by CIV-3 (capability handle entropy) | civ-maintainer | merged into CIV-3 | threat model TM-3 |
| CIV-3 | Capability handle entropy against cross-process guessing requires the process manager | civ-maintainer | open | threat model TM-3 |
| CIV-4 | PMC-backed benchmark receipts require isolated, frequency-locked hardware | civ-maintainer | open | threat model TM-3 |
| CIV-5 | Cascading revocation: parent revocation invalidating derived capabilities | civ-maintainer | open | capability crate docs |
| CEP-11 | Cyclic scan cursor for civ-pmm alloc: removes the wrap re-scan redundancy disclosed in the alloc OPTNOTE | civ-maintainer | open | civ-pmm alloc optimality note |
| CEP-12 | Fused run scan for civ-pmm find_zero_run: folds candidate verification into the skip loop | civ-maintainer | open | bitmap find_zero_run optimality note |
| CEP-13 | Free-list slot allocation for civ-capability alloc/derive: removes the O(capacity) first-fit scan | civ-maintainer | open | capability alloc/derive optimality notes |
| CEP-90 | cep_hunt per-commit Bloodhound infrastructure: diff-targeted fuzzing, differential execution, mutation testing, signed receipts | civ-maintainer | open | tests TEST-SCALING fields; threat model CIV-3 |
| CEP-91 | Benchmark receipt infrastructure on pinned, PMC-capable hardware per standards ch. 69 | civ-maintainer | open | every OPT-1 benchmark-pending note; threat model CIV-4 |
