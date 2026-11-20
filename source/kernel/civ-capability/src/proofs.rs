// CEP:FILE: source/kernel/civ-capability/src/proofs.rs
// CEP:WHAT: Kani bounded model-checking harnesses for the capability subsystem: attenuation-only derivation over the full rights space, fail-closed lookup over the full handle space, revocation-closes-access, total handle-space panic freedom, and the accounting invariant.
// CEP:WHY: Capability integrity is a mandatory design.md 21 system proof and threat model TM-3 is the declared adversary; the properties must be discharged mechanically over symbolic inputs, not by review comments (standards 54.1 FV prime law).
// CEP:CLASS: FV-1
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness. Harnesses are excluded from non-Kani builds by cfg(kani).
// CEP:ASSUMES: Bounded model: table capacity is fixed per harness (4 slots), generation counters are u32, per-harness unwind bounds are declared via #[kani::unwind] (annotated on every harness), and handle/ rights inputs are symbolic over their declared ranges. Bounds are recorded in verification/proof-registry.json; properties beyond the bound are covered by checked arithmetic plus tests, and this gap is the declared FV-1 refinement note.
// CEP:COST: proof-time only; cfg(kani) means zero instructions are linked into any kernel or test image.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json); registry entries in verification/proof-registry.json.
// CEP:SECURITY: Harnesses model the TM-3 adversary with kani::any over full u64 handle and rights ranges.
// CEP:FV-CLASS: FV-1
// CEP:FV-PROVER: Kani (exact version recorded in verification/evidence/toolchain.json at proof time)
// CEP:FV-SCRIPT: cargo kani -p civ-capability
// CEP:FV-SORRY-COUNT: 0
// CEP:FV-REFINEMENT: Harnesses exercise the exact kernel source functions (no abstract model). Bounded: 4-slot tables, u32 generations, single-step revocation semantics (cascading revocation is the declared crate subset, ticket CIV-5); unbounded universality is not claimed and is the declared FV-1 gap, mitigated by runtime checked arithmetic.
// CEP:FV-AXIOMS: none
//! Kani proof harnesses for the capability subsystem.
//!
//! Every harness runs against the same source the kernel ships. Bounds are
//! declared per harness and registered in verification/proof-registry.json.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::error::CapabilityError;
use crate::rights::{Rights, RIGHT_GRANT};
use crate::table::{CapabilityTable, Handle, SlotEntry};

/// Slot capacity used by every harness.
const CAPACITY: usize = 4;

/// Harness: derive creates a child only when the request is a subset of the
/// parent's rights and the parent holds the grant right; the child's rights
/// equal the requested subset on the parent's object.
///
/// Property (capability integrity, design.md 21): over the full u64 rights
/// space, no derive result ever holds a right the parent does not hold.
///
/// Model bound: 4-slot table, parent allocated first, u32 generations.
// CEP:ID: CS-a08
// CEP:WHAT: Kani harness proving derive never escalates: child rights equal the request, sit inside the parent set, and require the grant right, over symbolic u64 rights space.
// CEP:WHY: Capability integrity (design.md 21) and threat model TM-3 escalation refusal discharged over the full rights space; the classification biconditional covers Ok, GrantDenied, and Escalation exactly.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(8)]
fn proof_derive_never_escalates() {
    let parent_rights_raw: u64 = kani::any();
    let grant_flag: bool = kani::any();
    let requested_raw: u64 = kani::any();
    let parent_object: u64 = kani::any();

    let mut backing = [SlotEntry::empty(); CAPACITY];
    let mut table = CapabilityTable::new(&mut backing).unwrap();
    let parent_rights =
        Rights::from_raw(parent_rights_raw | if grant_flag { RIGHT_GRANT } else { 0 });
    let parent = table.alloc(parent_object, parent_rights).unwrap();
    let requested = Rights::from_raw(requested_raw);

    let result = table.derive(parent, requested);
    let grant_ok = grant_flag;
    let subset_ok = parent_rights.contains_all(requested);
    match result {
        Ok(child) => {
            assert!(
                grant_ok && subset_ok,
                "derive succeeded without grant or inside escalation"
            );
            let (child_rights, child_object) = table.lookup(child).unwrap();
            assert_eq!(child_rights.as_raw(), requested_raw);
            assert_eq!(child_object, parent_object);
            assert!(
                parent_rights.contains_all(child_rights),
                "child holds rights the parent does not hold"
            );
        }
        Err(CapabilityError::GrantDenied) => {
            assert!(!grant_ok, "GrantDenied requires a grant-less parent");
            assert_eq!(table.allocated_count(), 1);
        }
        Err(CapabilityError::Escalation) => {
            assert!(
                grant_ok && !subset_ok,
                "Escalation requires grant and a non-subset request"
            );
            assert_eq!(table.allocated_count(), 1);
        }
        Err(other) => panic!("unexpected error class: {other:?}"),
    }
}

/// Harness: lookup accepts exactly the handles that resolve to a live slot
/// with a matching generation and fails closed with a typed error for every
/// other u64 value.
///
/// Property (threat model TM-3 forgery): any u64 presented to lookup either
/// resolves to the correct capability or is rejected; no input panics.
///
/// Model bound: 4-slot table with symbolic occupancy of the first two slots.
// CEP:ID: CS-a09
// CEP:WHAT: Kani harness proving lookup resolves exactly the live handles and fails closed with typed errors over the full u64 handle space.
// CEP:WHY: Threat model TM-3 forgery discharge: any u64 either resolves to the correct capability contents or is rejected as InvalidSlot, SlotEmpty, or StaleHandle.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(8)]
fn proof_lookup_fail_closed() {
    let object_a: u64 = kani::any();
    let object_b: u64 = kani::any();
    let rights_a_raw: u64 = kani::any();
    let rights_b_raw: u64 = kani::any();
    let mut backing = [SlotEntry::empty(); CAPACITY];
    let mut table = CapabilityTable::new(&mut backing).unwrap();
    let handle_a = table
        .alloc(object_a, Rights::from_raw(rights_a_raw))
        .unwrap();
    let handle_b = table
        .alloc(object_b, Rights::from_raw(rights_b_raw))
        .unwrap();

    let target: Handle = Handle::pack(kani::any(), kani::any());
    let result = table.lookup(target);

    let slot = target.slot_index() as usize;
    let live_gen_a = handle_a.generation();
    if target == handle_a {
        let (rights, object) = result.unwrap();
        assert_eq!(rights.as_raw(), rights_a_raw);
        assert_eq!(object, object_a);
    } else if target == handle_b {
        let (rights, object) = result.unwrap();
        assert_eq!(rights.as_raw(), rights_b_raw);
        assert_eq!(object, object_b);
    } else {
        let classified = matches!(
            result,
            Err(CapabilityError::InvalidSlot)
                | Err(CapabilityError::SlotEmpty)
                | Err(CapabilityError::StaleHandle)
        );
        assert!(
            classified,
            "every non-live handle must fail closed with a typed error"
        );
        let _ = slot;
        let _ = live_gen_a;
    }
}

/// Harness: after a successful revoke, the revoked handle never resolves
/// again, re-revocation is rejected, and a reused slot's new handle is
/// distinct from the old one with the old handle still failing closed.
///
/// Property (design.md 21 revocation; crate invariant 3).
///
/// Model bound: 4-slot table, single allocation cycle.
// CEP:ID: CS-a10
// CEP:WHAT: Kani harness proving the full revocation lifecycle: closure, double-revoke rejection, slot reuse with generation advance, and stale-handle closure after reuse.
// CEP:WHY: Crate invariant 3 and design.md 21 revocation proof; the generation-advance mechanism is exercised end to end so reuse cannot revive old handles.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(8)]
fn proof_revoke_closes_access() {
    let object: u64 = kani::any();
    let rights = Rights::from_raw(kani::any());
    let mut backing = [SlotEntry::empty(); CAPACITY];
    let mut table = CapabilityTable::new(&mut backing).unwrap();
    let handle = table.alloc(object, rights).unwrap();
    assert!(table.lookup(handle).is_ok());

    table.revoke(handle).unwrap();
    assert!(
        matches!(table.lookup(handle), Err(CapabilityError::SlotEmpty)),
        "revoked handle must not resolve after the slot frees"
    );
    assert!(matches!(
        table.revoke(handle),
        Err(CapabilityError::SlotEmpty),
        "double revoke must be rejected"
    ));

    let reused = table.alloc(object, rights).unwrap();
    assert_eq!(
        reused.slot_index(),
        handle.slot_index(),
        "first-fit must reuse the freed slot"
    );
    assert_ne!(
        reused.generation(),
        handle.generation(),
        "reused slot must carry an advanced generation"
    );
    assert!(
        matches!(table.lookup(handle), Err(CapabilityError::StaleHandle)),
        "the old handle must stay closed after slot reuse"
    );
    assert!(table.lookup(reused).is_ok());
    assert_eq!(table.allocated_count(), 1);
    assert_eq!(table.allocated_popcount(), 1);
}

/// Harness: every u64 handle is total against lookup, revoke, and
/// rights_of; each returns a typed classification and no input panics.
///
/// Property (threat model TM-2/TM-3 fail-closed totality).
///
/// Model bound: 4-slot table with one live capability.
// CEP:ID: CS-a11
// CEP:WHAT: Kani harness proving lookup, rights_of, and revoke are total over the full u64 handle space with typed classifications and no panic.
// CEP:WHY: Threat model TM-2 fail-closed totality for the authority boundary; includes post-revoke closure of the live handle.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(8)]
fn proof_handle_space_no_panic() {
    let mut backing = [SlotEntry::empty(); CAPACITY];
    let mut table = CapabilityTable::new(&mut backing).unwrap();
    let live = table.alloc(0x1234, Rights::all()).unwrap();

    let target: Handle = Handle::pack(kani::any(), kani::any());
    let lookup_result = table.lookup(target);
    let rights_result = table.rights_of(target);
    let revoke_result = table.revoke(target);

    if target == live {
        assert!(lookup_result.is_ok());
        assert!(rights_result.is_ok());
        assert!(revoke_result.is_ok());
        // After the successful revoke the same handle is closed forever.
        assert!(matches!(
            table.lookup(target),
            Err(CapabilityError::SlotEmpty)
        ));
    } else {
        assert!(
            matches!(
                lookup_result,
                Err(CapabilityError::InvalidSlot)
                    | Err(CapabilityError::SlotEmpty)
                    | Err(CapabilityError::StaleHandle)
            ),
            "lookup must fail closed"
        );
        assert!(
            matches!(
                rights_result,
                Err(CapabilityError::InvalidSlot)
                    | Err(CapabilityError::SlotEmpty)
                    | Err(CapabilityError::StaleHandle)
            ),
            "rights_of must fail closed"
        );
        assert!(
            matches!(
                revoke_result,
                Err(CapabilityError::InvalidSlot)
                    | Err(CapabilityError::SlotEmpty)
                    | Err(CapabilityError::StaleHandle)
            ),
            "revoke must fail closed"
        );
    }
}

/// Harness: the accounting counter, the slot popcount, and the capacity
/// subtraction agree after every operation in a nondeterministic sequence.
///
/// Property (crate invariant: allocated == popcount <= capacity).
///
/// Model bound: 4-slot table; four nondeterministically chosen operations.
// CEP:ID: CS-a12
// CEP:WHAT: Kani harness proving the accounting counter, slot popcount, and capacity subtraction agree after every operation in a nondeterministic sequence.
// CEP:WHY: Crate accounting invariant under symbolic operation interleavings within the 4-slot bound; mirrors the frame allocator invariant discipline.
// CEP:STATUS: complete
// CEP:FAILURE: A violated property aborts cargo kani with a counterexample; a broken harness precondition panics the harness.
// CEP:ASSUMES: Declared model bound (file header FV-REFINEMENT and verification/proof-registry.json).
// CEP:COST: proof-time only; cfg(kani) excludes harnesses from all non-Kani builds.
// CEP:EVIDENCE: cargo kani -p civ-capability; results UNVERIFIED until a recorded run exists (status in verification/evidence/toolchain.json).
#[kani::proof]
#[kani::unwind(16)]
fn proof_invariant_slots_consistent() {
    let mut backing = [SlotEntry::empty(); CAPACITY];
    let mut table = CapabilityTable::new(&mut backing).unwrap();
    let mut live: Option<Handle> = None;

    let mut step = 0;
    while step < 4 {
        let operation: u8 = kani::any();
        match operation % 3 {
            0 => {
                if live.is_none() {
                    if let Ok(handle) = table.alloc(0, Rights::all()) {
                        live = Some(handle);
                    }
                }
            }
            1 => {
                if let Some(handle) = live {
                    assert!(table.revoke(handle).is_ok());
                    live = None;
                }
            }
            _ => {
                assert_eq!(
                    table.allocated_count(),
                    table.allocated_popcount(),
                    "accounting drifted from slot occupancy"
                );
                assert!(table.allocated_count() <= table.capacity());
                assert_eq!(
                    table.allocated_count() + table.free_count(),
                    table.capacity()
                );
            }
        }
        step += 1;
    }
    assert_eq!(
        table.allocated_count(),
        table.allocated_popcount(),
        "final state drifted from slot occupancy"
    );
}
