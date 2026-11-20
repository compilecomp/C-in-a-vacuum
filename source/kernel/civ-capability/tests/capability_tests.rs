// CEP:FILE: source/kernel/civ-capability/tests/capability_tests.rs
// CEP:WHAT: Adversarial-dominant unit and property test suite for the capability subsystem.
// CEP:WHY: Standards 61.1 and 61.4 require worst-case-dominant coverage; design.md 9.1 and threat model TM-3 make escalation refusal, forgery resistance, and revocation the properties that must be pinned before the kernel trusts this boundary.
// CEP:CLASS: CC
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion fails the cargo test run and blocks CI; no test mutates shared state.
// CEP:ASSUMES: Tests run on the host target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline; runtime cost irrelevant (CEP-2-class evidence generation).
// CEP:EVIDENCE: cargo test -p civ-capability output recorded in verification/evidence/test-run.log.
// CEP:SECURITY: Scenarios model threat-model TM-3 adversaries: forged handles, stale generations, escalation requests, grant-less parents.
// CEP:TEST-CLASS: Unit + property (adversarial-dominant)
// CEP:TEST-TARGET: civ-capability rights bitset, capability table, handle encoding
// CEP:TEST-QUOTA: 26 scenarios per change; Kani bounded proofs per proofs.rs (5 harnesses); targeted fuzz harness deferred to the cep_hunt infrastructure (ticket CEP-90), with Kani bounded proofs as the declared substitute within their bounds
// CEP:TEST-DISTRIBUTION: 14 worst-case/adversarial (54%), 11 middle (42%), 1 happy (4%)
// CEP:TEST-SCALING: Every new failure mode adds its regression matrix (standards 61.3, 50 tests per confirmed bug)
// CEP:TEST-EVIDENCE: cargo test -p civ-capability output recorded in verification/evidence/test-run.log

//! Adversarial-dominant tests for the capability subsystem.

use civ_capability::{
    CapabilityError, CapabilityTable, Handle, Rights, SlotEntry, RIGHT_EXECUTE, RIGHT_GRANT,
    RIGHT_MAP, RIGHT_READ, RIGHT_WRITE,
};

/// Builds a table over `slots` fresh slots.
// CEP:WHAT: Builds a fresh capability table over a leaked backing store of the requested size.
// CEP:WHY: Shared test scaffolding; the deliberate leak stands in for kernel-owned table memory while the code under test stays allocation-free.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability build; results recorded in verification/evidence/test-run.log.
fn build(slots: usize) -> CapabilityTable<'static> {
    let backing: &'static mut [SlotEntry] =
        Box::leak(vec![SlotEntry::empty(); slots].into_boxed_slice());
    CapabilityTable::new(backing).expect("test configuration must be structurally valid")
}

// ---------------------------------------------------------------------------
// Construction validation (worst-case/adversarial)
// ---------------------------------------------------------------------------

#[test]
// CEP:WHAT: Rejects construction over a zero-length backing store.
// CEP:WHY: Guards the InvalidArgument classification; a zero-slot table cannot host any capability and must fail closed at construction.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_table_rejects_empty_backing; results recorded in verification/evidence/test-run.log.
fn test_table_rejects_empty_backing() {
    let mut backing: [SlotEntry; 0] = [];
    assert_eq!(
        CapabilityTable::new(&mut backing).unwrap_err(),
        CapabilityError::InvalidArgument
    );
}

#[test]
// CEP:WHAT: Reuses backing memory that held live capabilities and verifies the new table sees an empty state.
// CEP:WHY: Guards the no-phantom-capability obligation (design.md 4.5 analog): prior contents must never surface as live authority.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_table_initializes_backing; results recorded in verification/evidence/test-run.log.
fn test_table_initializes_backing() {
    // Reused backing memory with live capabilities must arrive initialized:
    // no prior contents may surface as live capabilities in the new table
    // (design.md 4.5 no uninitialized disclosure).
    let backing: &'static mut [SlotEntry] =
        Box::leak(vec![SlotEntry::empty(); 4].into_boxed_slice());
    {
        let mut first = CapabilityTable::new(backing).unwrap();
        for step in 0..4u64 {
            first.alloc(step, Rights::all()).unwrap();
        }
        assert_eq!(first.allocated_count(), 4);
    }
    let second = CapabilityTable::new(backing).unwrap();
    assert_eq!(second.allocated_count(), 0);
    assert_eq!(second.free_count(), 4);
    assert_eq!(second.allocated_popcount(), 0);
}

#[test]
// CEP:WHAT: Verifies the handle packing round-trip at known bit patterns.
// CEP:WHY: Pins the slot/generation encoding in one place so forging resistance has a stable target.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_handle_roundtrip_encoding; results recorded in verification/evidence/test-run.log.
fn test_handle_roundtrip_encoding() {
    let handle = Handle::pack(0x1234_5678, 0x9ABC_DEF0);
    assert_eq!(handle.slot_index(), 0x1234_5678);
    assert_eq!(handle.generation(), 0x9ABC_DEF0);
    assert_eq!(handle.as_raw(), 0x9ABC_DEF0_1234_5678);
}

// ---------------------------------------------------------------------------
// Allocation (worst-case exhaustion)
// ---------------------------------------------------------------------------

#[test]
// CEP:WHAT: Allocates one capability and resolves it to the exact rights and object stored.
// CEP:WHY: Content-checking allocation scenario for the accounting state machine (middle class; the suite's single happy smoke is test_full_lifecycle).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_alloc_returns_usable_handle; results recorded in verification/evidence/test-run.log.
fn test_alloc_returns_usable_handle() {
    let mut table = build(4);
    let handle = table
        .alloc(0x42, Rights::from_raw(RIGHT_READ | RIGHT_WRITE))
        .unwrap();
    let (rights, object) = table.lookup(handle).unwrap();
    assert_eq!(rights.as_raw(), RIGHT_READ | RIGHT_WRITE);
    assert_eq!(object, 0x42);
}

#[test]
// CEP:WHAT: Allocates every slot, then verifies TableFull at exactly the boundary.
// CEP:WHY: Guards the exhaustion classification and exact accounting at the full table.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_table_exhaustion_exact; results recorded in verification/evidence/test-run.log.
fn test_table_exhaustion_exact() {
    let mut table = build(4);
    for step in 0..4 {
        assert!(
            table.alloc(step, Rights::all()).is_ok(),
            "step {step} must allocate"
        );
    }
    assert!(matches!(
        table.alloc(9, Rights::all()),
        Err(CapabilityError::TableFull)
    ));
    assert_eq!(table.allocated_count(), 4);
}

#[test]
// CEP:WHAT: Allocates eight capabilities and verifies all slot indexes are distinct.
// CEP:WHY: Guards no-double-allocation across the first-fit scan.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_alloc_distinct_slots; results recorded in verification/evidence/test-run.log.
fn test_alloc_distinct_slots() {
    let mut table = build(8);
    let mut slots: Vec<u32> = Vec::new();
    for step in 0..8u64 {
        slots.push(table.alloc(step, Rights::none()).unwrap().slot_index());
    }
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), 8, "slots must be distinct");
}

#[test]
// CEP:WHAT: Verifies counter, popcount, and capacity sum agree after every allocation step.
// CEP:WHY: Crate accounting invariant observed across a workload.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_allocated_count_consistency; results recorded in verification/evidence/test-run.log.
fn test_allocated_count_consistency() {
    let mut table = build(6);
    for step in 0..6u64 {
        table.alloc(step, Rights::all()).unwrap();
        assert_eq!(table.allocated_count(), step + 1);
        assert_eq!(table.allocated_popcount(), step + 1);
        assert_eq!(
            table.allocated_count() + table.free_count(),
            table.capacity()
        );
    }
}

// ---------------------------------------------------------------------------
// Derivation (the security core: escalation, grants, attenuation)
// ---------------------------------------------------------------------------

#[test]
// CEP:WHAT: Derives a read-only child from a read-write parent with grant and verifies exact child contents.
// CEP:WHY: The attenuation contract (design.md 9.1): child equals request, inherits object.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_attenuates; results recorded in verification/evidence/test-run.log.
fn test_derive_attenuates() {
    let mut table = build(4);
    let parent = table
        .alloc(
            0x100,
            Rights::from_raw(RIGHT_READ | RIGHT_WRITE | RIGHT_GRANT),
        )
        .unwrap();
    let child = table.derive(parent, Rights::from_raw(RIGHT_READ)).unwrap();
    let (rights, object) = table.lookup(child).unwrap();
    assert_eq!(rights.as_raw(), RIGHT_READ);
    assert_eq!(object, 0x100, "child inherits the parent's object");
}

#[test]
// CEP:WHAT: Verifies a write request over a read-only parent is refused as Escalation with state unchanged.
// CEP:WHY: Guards the capability-integrity core (design.md 21, TM-3).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_refuses_escalation; results recorded in verification/evidence/test-run.log.
fn test_derive_refuses_escalation() {
    let mut table = build(4);
    let parent = table
        .alloc(0x100, Rights::from_raw(RIGHT_READ | RIGHT_GRANT))
        .unwrap();
    let result = table.derive(parent, Rights::from_raw(RIGHT_READ | RIGHT_WRITE));
    assert!(matches!(result, Err(CapabilityError::Escalation)));
    // Refused derive must leave the table unchanged.
    assert_eq!(table.allocated_count(), 1);
    assert_eq!(table.allocated_popcount(), 1);
}

#[test]
// CEP:WHAT: Verifies derivation from a grant-less parent is refused as GrantDenied.
// CEP:WHY: Separates use authority from delegation authority (design.md 9.1 transfer).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_requires_grant_right; results recorded in verification/evidence/test-run.log.
fn test_derive_requires_grant_right() {
    let mut table = build(4);
    let parent = table.alloc(0x100, Rights::from_raw(RIGHT_READ)).unwrap();
    assert!(matches!(
        table.derive(parent, Rights::from_raw(RIGHT_READ)),
        Err(CapabilityError::GrantDenied)
    ));
}

#[test]
// CEP:WHAT: Verifies derive with out-of-range, unallocated, and stale parents is refused as ParentInvalid.
// CEP:WHY: All three parent-resolution failure modes must classify identically and mutate nothing.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_invalid_parent_fails_closed; results recorded in verification/evidence/test-run.log.
fn test_derive_invalid_parent_fails_closed() {
    let mut table = build(4);
    // Out-of-range slot index.
    let oob = Handle::pack(99, 0);
    assert!(matches!(
        table.derive(oob, Rights::all()),
        Err(CapabilityError::ParentInvalid)
    ));
    // Valid slot index but never allocated.
    let empty_slot = Handle::pack(1, 0);
    assert!(matches!(
        table.derive(empty_slot, Rights::all()),
        Err(CapabilityError::ParentInvalid)
    ));
    // Live slot but wrong generation.
    let parent = table.alloc(0x1, Rights::all()).unwrap();
    let stale = Handle::pack(parent.slot_index(), parent.generation().wrapping_add(1));
    assert!(matches!(
        table.derive(stale, Rights::all()),
        Err(CapabilityError::ParentInvalid)
    ));
    assert_eq!(table.allocated_count(), 1);
}

#[test]
// CEP:WHAT: Verifies attenuation to the empty rights set succeeds and produces a do-nothing capability.
// CEP:WHY: Attenuate-to-nothing is legal; only escalation is refused (design.md 9.1).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_empty_rights_legal; results recorded in verification/evidence/test-run.log.
fn test_derive_empty_rights_legal() {
    // Attenuation to nothing is legal; escalation is not (design.md 9.1).
    let mut table = build(4);
    let parent = table
        .alloc(0x7, Rights::from_raw(RIGHT_READ | RIGHT_GRANT))
        .unwrap();
    let child = table.derive(parent, Rights::none()).unwrap();
    let (rights, _) = table.lookup(child).unwrap();
    assert_eq!(rights.as_raw(), 0);
}

#[test]
// CEP:ID: CS-a13
// CEP:WHAT: Exhaustive sweep over every (parent rights, request rights) pair of the five named rights with and without the grant flag, asserting classification and child contents for all 2048 combinations (64 parents x 32 requests).
// CEP:WHY: The named-rights space is checked exhaustively rather than sampled, so a masking regression in any single right bit cannot hide; unit counterpart of the Kani escalation proof.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_full_space_never_escalates; results recorded in verification/evidence/test-run.log.
fn test_derive_full_space_never_escalates() {
    // Exhaustive sweep over the five named rights: every (parent, request)
    // pair with the grant flag either succeeds inside the parent set or is
    // refused; a succeeded child never exceeds its parent. This is the
    // unit-test counterpart of the Kani escalation proof over u64.
    let named = [
        RIGHT_READ,
        RIGHT_WRITE,
        RIGHT_EXECUTE,
        RIGHT_MAP,
        RIGHT_GRANT,
    ];
    let mut table = build(1 << 11);
    let mut successes = 0u64;
    for parent_mask in 0..32u64 {
        for grant in [false, true] {
            let mut raw = 0u64;
            for bit in 0..5u64 {
                if parent_mask & (1 << bit) != 0 {
                    raw |= named[bit as usize];
                }
            }
            if grant {
                raw |= RIGHT_GRANT;
            }
            let parent = table.alloc(parent_mask, Rights::from_raw(raw)).unwrap();
            let _ = parent;
            for request_mask in 0..32u64 {
                let mut req = 0u64;
                for bit in 0..5u64 {
                    if request_mask & (1 << bit) != 0 {
                        req |= named[bit as usize];
                    }
                }
                let result = table.derive(parent, Rights::from_raw(req));
                let subset = raw & req == req;
                let grant_ok = raw & RIGHT_GRANT != 0;
                match result {
                    Ok(child) => {
                        assert!(subset && grant_ok);
                        successes += 1;
                        let child_rights = table.rights_of(child).unwrap();
                        assert_eq!(child_rights.as_raw(), req);
                        assert_eq!(raw & child_rights.as_raw(), child_rights.as_raw());
                    }
                    Err(CapabilityError::Escalation) => assert!(grant_ok && !subset),
                    Err(CapabilityError::GrantDenied) => assert!(!grant_ok),
                    Err(other) => panic!("unexpected error: {other:?}"),
                }
            }
        }
    }
    // 64 parents plus every successful derive; exact accounting.
    assert_eq!(table.allocated_count(), 64 + successes);
    assert_eq!(table.allocated_popcount(), 64 + successes);
}

#[test]
// CEP:WHAT: Verifies a refused derive leaves counter and popcount unchanged.
// CEP:WHY: Refusal atomicity: no half-marked slots after Escalation (standards 66.7).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_state_unchanged_on_error; results recorded in verification/evidence/test-run.log.
fn test_derive_state_unchanged_on_error() {
    let mut table = build(4);
    let parent = table
        .alloc(0x5, Rights::from_raw(RIGHT_READ | RIGHT_GRANT))
        .unwrap();
    let before_count = table.allocated_count();
    let before_pop = table.allocated_popcount();
    assert!(matches!(
        table.derive(parent, Rights::from_raw(RIGHT_WRITE | RIGHT_EXECUTE)),
        Err(CapabilityError::Escalation)
    ));
    assert_eq!(table.allocated_count(), before_count);
    assert_eq!(table.allocated_popcount(), before_pop);
}

// ---------------------------------------------------------------------------
// Revocation and lookup (worst-case forgery)
// ---------------------------------------------------------------------------

#[test]
// CEP:WHAT: Verifies revocation closes resolution, double revocation is rejected, and the classification is SlotEmpty.
// CEP:WHY: Crate invariant 3 and design.md 21 revocation at unit speed.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_revoke_closes_access; results recorded in verification/evidence/test-run.log.
fn test_revoke_closes_access() {
    let mut table = build(4);
    let handle = table.alloc(0x9, Rights::all()).unwrap();
    assert!(table.lookup(handle).is_ok());
    table.revoke(handle).unwrap();
    assert!(matches!(
        table.lookup(handle),
        Err(CapabilityError::SlotEmpty)
    ));
    assert!(matches!(
        table.revoke(handle),
        Err(CapabilityError::SlotEmpty)
    ));
}

#[test]
// CEP:WHAT: Verifies revoking an out-of-range handle leaves counter and popcount unchanged.
// CEP:WHY: No-mutation on rejection for the revocation path.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_revoke_state_unchanged_on_error; results recorded in verification/evidence/test-run.log.
fn test_revoke_state_unchanged_on_error() {
    let mut table = build(4);
    let handle = table.alloc(0x3, Rights::all()).unwrap();
    let oob = Handle::pack(77, 0);
    assert!(matches!(
        table.revoke(oob),
        Err(CapabilityError::InvalidSlot)
    ));
    assert_eq!(table.allocated_count(), 1);
    assert_eq!(table.allocated_popcount(), 1);
    let _ = handle;
}

#[test]
// CEP:WHAT: Verifies out-of-range and unallocated handles are rejected with the correct typed classification.
// CEP:WHY: The lookup gate's classification surface (TM-3).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_lookup_fail_closed; results recorded in verification/evidence/test-run.log.
fn test_lookup_fail_closed() {
    let mut table = build(4);
    table.alloc(0x1, Rights::all()).unwrap();
    // Out-of-range slot indexes.
    for slot in [4u32, 5, 0xFFFF_FFFF] {
        assert!(matches!(
            table.lookup(Handle::pack(slot, 0)),
            Err(CapabilityError::InvalidSlot)
        ));
    }
    // In-range but empty slot.
    assert!(matches!(
        table.lookup(Handle::pack(2, 0)),
        Err(CapabilityError::SlotEmpty)
    ));
}

#[test]
// CEP:WHAT: Verifies a pre-revocation handle stays closed after slot reuse and the new handle resolves.
// CEP:WHY: Generation advance is the mechanism that keeps reuse from reviving old handles.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_stale_handle_fails_closed; results recorded in verification/evidence/test-run.log.
fn test_stale_handle_fails_closed() {
    let mut table = build(4);
    let handle = table.alloc(0x2, Rights::all()).unwrap();
    table.revoke(handle).unwrap();
    let reused = table.alloc(0x2, Rights::all()).unwrap();
    assert_ne!(reused.generation(), handle.generation());
    assert!(matches!(
        table.lookup(handle),
        Err(CapabilityError::StaleHandle)
    ));
    assert!(table.lookup(reused).is_ok());
}

#[test]
// CEP:WHAT: Verifies generation values one above and one below the live generation are rejected as StaleHandle.
// CEP:WHY: Adjacent-generation forgery, the cheapest adversary probe, must fail.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_forged_generation_fails; results recorded in verification/evidence/test-run.log.
fn test_forged_generation_fails() {
    let mut table = build(4);
    let handle = table.alloc(0x4, Rights::all()).unwrap();
    let forged = Handle::pack(handle.slot_index(), handle.generation().wrapping_add(1));
    assert!(matches!(
        table.lookup(forged),
        Err(CapabilityError::StaleHandle)
    ));
    let forged_low = Handle::pack(handle.slot_index(), handle.generation().wrapping_sub(1));
    assert!(matches!(
        table.lookup(forged_low),
        Err(CapabilityError::StaleHandle)
    ));
}

#[test]
// CEP:ID: CS-a14
// CEP:WHAT: Flips each of the 64 bits of a live handle and verifies only the unmutated handle resolves.
// CEP:WHY: Maximum-entropy forgery sweep pinning bit-level handle integrity at unit speed.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_handle_high_bits_forgery; results recorded in verification/evidence/test-run.log.
fn test_handle_high_bits_forgery() {
    // Maximum-entropy forged handles: every bit of the u64 flipped from a
    // live handle must fail closed.
    let mut table = build(4);
    let handle = table.alloc(0x6, Rights::all()).unwrap();
    let raw = handle.as_raw();
    for bit in 0..64u64 {
        let forged_raw = raw ^ (1u64 << bit);
        let forged = Handle::pack((forged_raw & 0xFFFF_FFFF) as u32, (forged_raw >> 32) as u32);
        let result = table.lookup(forged);
        if forged_raw == raw {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err(), "bit {bit} flip must not resolve");
        }
    }
}

#[test]
// CEP:WHAT: Verifies the five named rights are distinct powers of two and the intersection predicates classify correctly.
// CEP:WHY: The rights algebra underpins every attenuation check; distinctness is its first axiom.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct; results recorded in verification/evidence/test-run.log.
fn test_rights_named_bits_distinct() {
    let named = [
        RIGHT_READ,
        RIGHT_WRITE,
        RIGHT_EXECUTE,
        RIGHT_MAP,
        RIGHT_GRANT,
    ];
    for (i, a) in named.iter().enumerate() {
        for (j, b) in named.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "named rights must be distinct powers of two");
            }
        }
    }
    assert!(Rights::all().intersects(Rights::from_raw(RIGHT_READ)));
    assert!(!Rights::none().intersects(Rights::from_raw(RIGHT_READ)));
}

#[test]
// CEP:WHAT: Verifies attenuation to the empty set produces a do-nothing capability that stays inside the parent.
// CEP:WHY: Attenuate-to-nothing legality with subset maintenance.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_none_attenuation_legal; results recorded in verification/evidence/test-run.log.
fn test_rights_none_attenuation_legal() {
    let full = Rights::all();
    let attenuated = full.attenuate(Rights::none());
    assert_eq!(attenuated.as_raw(), 0);
    assert!(!attenuated.intersects(full));
    assert!(full.contains_all(attenuated));
}

#[test]
// CEP:WHAT: Verifies raw bitset round-trips at boundary values including u64::MAX.
// CEP:WHY: The rights type is total over u64; boundary values must survive the wrap.
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_from_raw_roundtrip; results recorded in verification/evidence/test-run.log.
fn test_rights_from_raw_roundtrip() {
    for raw in [
        0u64,
        1,
        RIGHT_READ | RIGHT_MAP,
        u64::MAX,
        0x8000_0000_0000_0000,
    ] {
        assert_eq!(Rights::from_raw(raw).as_raw(), raw);
    }
    assert!(Rights::from_raw(u64::MAX).contains_all(Rights::all()));
}

#[test]
// CEP:WHAT: Verifies Display names exactly the held rights, renders none for empty, and never drops unnamed bits.
// CEP:WHY: Audit-log faithfulness (design.md 9.1 authority audit; standards 66.1).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_display_names_held_rights; results recorded in verification/evidence/test-run.log.
fn test_rights_display_names_held_rights() {
    let rendered = Rights::from_raw(RIGHT_READ | RIGHT_GRANT).to_string();
    assert_eq!(rendered, "read+grant");
    assert_eq!(Rights::none().to_string(), "none");
    // Unnamed bits are rendered, never silently dropped (standards 66.1).
    let unnamed = Rights::from_raw(1u64 << 63).to_string();
    assert!(unnamed.contains("unnamed"), "{unnamed}");
}

#[test]
// CEP:WHAT: Verifies every error variant renders non-empty, deterministic messages.
// CEP:WHY: Diagnostics stability without allocation (standards 25.3.1).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_capability_error_display; results recorded in verification/evidence/test-run.log.
fn test_capability_error_display() {
    let samples = [
        CapabilityError::InvalidArgument,
        CapabilityError::InvalidSlot,
        CapabilityError::SlotEmpty,
        CapabilityError::StaleHandle,
        CapabilityError::TableFull,
        CapabilityError::Escalation,
        CapabilityError::ParentInvalid,
        CapabilityError::GrantDenied,
    ];
    for error in samples {
        let rendered = error.to_string();
        assert!(!rendered.is_empty());
        assert_eq!(rendered, error.to_string());
    }
}

// ---------------------------------------------------------------------------
// Happy-path smoke (1 scenario, < 5% per standards 61.4.1)
// ---------------------------------------------------------------------------

#[test]
// CEP:WHAT: Allocates, derives, uses, revokes the child, and revokes the parent, verifying accounting at each step.
// CEP:WHY: Happy-path smoke: the minimal valid-input coverage the Reality Curve permits (standards 61.4.1).
// CEP:STATUS: complete
// CEP:FAILURE: A failing assertion aborts the cargo test run; no state escapes the harness.
// CEP:ASSUMES: Host test target; test-only std allocation for scaffolding, none in the code under test.
// CEP:COST: offline test execution; runtime cost irrelevant to the kernel image.
// CEP:EVIDENCE: cargo test -p civ-capability test_full_lifecycle; results recorded in verification/evidence/test-run.log.
fn test_full_lifecycle() {
    let mut table = build(4);
    let parent = table
        .alloc(
            0x1000,
            Rights::from_raw(RIGHT_READ | RIGHT_WRITE | RIGHT_GRANT),
        )
        .unwrap();
    let child = table.derive(parent, Rights::from_raw(RIGHT_READ)).unwrap();
    assert!(table.lookup(child).is_ok());
    table.revoke(child).unwrap();
    assert!(table.lookup(child).is_err());
    table.revoke(parent).unwrap();
    assert_eq!(table.allocated_count(), 0);
}
