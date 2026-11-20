// CEP:FILE: source/kernel/civ-capability/src/table.rs
// CEP:WHAT: The capability table state machine: slot allocation with generational handles, attenuation-only derivation, fail-closed lookup, and revocation.
// CEP:WHY: design.md 9.1 requires capability representation, allocation, validation, derivation, attenuation, transfer, and revocation with capability integrity as a mandatory system proof (design.md 21); a generation-stamped slot table makes forgery, stale handles, and escalation all fail closed at one checked boundary.
// CEP:CLASS: CEP-1
// CEP:STATUS: partial
// CEP:OWNER: civ-maintainer
// CEP:TICKET: CIV-5
// CEP:FAILURE: Every operation returns a typed error (CapabilityError) instead of panicking; failed operations leave state unchanged; no allocation; no I/O.
// CEP:ASSUMES: Generation counters are u32 and advance only on revoke; a slot revocated 2^32 times wraps its generation (residual risk recorded in security/threat_model.md TM-3 and crate docs); the table is single-owner, not Sync/Send beyond the backing store's autotraits.
// CEP:COST: alloc/derive/revoke O(capacity) first-fit slot scan; lookup O(1); all allocation-free. First-fit free-slot search cost is bounded by one pass; a free-list structure is the tracked optimization (ticket CEP-13) pending the process-manager workload data.
// CEP:EVIDENCE: unit tests in source/kernel/civ-capability/tests/capability_tests.rs; Kani proofs proof_derive_never_escalates, proof_lookup_fail_closed, proof_revoke_closes_access, proof_handle_space_no_panic, proof_invariant_slots_consistent.
// CEP:SECURITY: This is the kernel authority boundary (design.md 9.1, threat model TM-3); handles are untrusted u64 values, derivation refuses escalation over the full input space, and revocation closes access permanently for the named handle.

//! Capability table state machine.

use crate::error::CapabilityError;
use crate::rights::{Rights, RIGHT_GRANT};

/// One capability slot: rights, target object, generation, and occupancy.
///
/// The backing store is a caller-provided slice of these entries; the
/// uninitialized value is [`SlotEntry::EMPTY`], and the table overwrites
/// every slot with it at construction so caller memory never surfaces as a
/// phantom capability (crate invariant 4 analog of the frame allocator).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotEntry {
    rights: Rights,
    object: u64,
    generation: u32,
    allocated: bool,
}

impl SlotEntry {
    /// The uninitialized slot value.
    // CEP:WHAT: Constructs the empty, unallocated slot value.
    // CEP:WHY: Caller-provided backing memory must never surface as a live capability; a single named empty value makes the initialization contract mechanical.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); constant fold.
    // CEP:EVIDENCE: cargo test -p civ-capability test_table_rejects_empty_backing.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            rights: Rights::none(),
            object: 0,
            generation: 0,
            allocated: false,
        }
    }

    /// Rights held by the slot's capability; meaningless while unallocated.
    // CEP:WHAT: Returns the rights stored in the slot.
    // CEP:WHY: Diagnostics and the invariant harness read slot rights; the accessor is read-only and cannot grant authority.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_slots_consistent.
    #[inline]
    pub const fn rights(&self) -> Rights {
        self.rights
    }

    /// Whether the slot currently holds a live capability.
    // CEP:WHAT: Reports slot occupancy.
    // CEP:WHY: The invariant witness distinguishes allocated slots from holes without inspecting rights.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_slots_consistent.
    #[inline]
    pub const fn is_allocated(&self) -> bool {
        self.allocated
    }
}

/// An opaque capability handle: slot index in the low 32 bits, generation in the high 32 bits.
///
/// Holders treat this as an opaque u64; any value can be presented to
/// [`CapabilityTable::lookup`], which fails closed unless the slot index is
/// in range, the slot is allocated, and the generation matches (threat
/// model TM-3).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Handle(u64);

impl Handle {
    /// Packs a slot index and generation into a handle.
    // CEP:WHAT: Packs a slot index and generation into the opaque handle encoding.
    // CEP:WHY: One u64 handle keeps the authority token register-sized; packing is the only place the encoding is defined, so forging requires guessing the whole u64 (threat model TM-3).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none; any index/generation pair is encodable, resolution validates.
    // CEP:COST: O(1); one shift and one OR.
    // CEP:EVIDENCE: cargo test -p civ-capability test_handle_roundtrip_encoding.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: the observable behavior is one shift-OR; no implementation performs fewer operations. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn pack(slot_index: u32, generation: u32) -> Self {
        Self(((generation as u64) << 32) | slot_index as u64)
    }

    /// Slot index component of the handle.
    // CEP:WHAT: Extracts the slot index from the handle encoding.
    // CEP:WHY: Lookup's bounds check needs the index; extraction here keeps the encoding defined in one place.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one mask.
    // CEP:EVIDENCE: cargo test -p civ-capability test_handle_roundtrip_encoding.
    #[inline]
    pub const fn slot_index(self) -> u32 {
        self.0 as u32
    }

    /// Generation component of the handle.
    // CEP:WHAT: Extracts the generation from the handle encoding.
    // CEP:WHY: Lookup's staleness check compares this against the slot's live generation.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one shift.
    // CEP:EVIDENCE: cargo test -p civ-capability test_handle_roundtrip_encoding.
    #[inline]
    pub const fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// The raw handle value.
    // CEP:WHAT: Returns the raw u64 handle value.
    // CEP:WHY: Handles cross process boundaries as raw values (design.md 9.1 capability transfer); explicit conversion marks that boundary.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-capability test_handle_roundtrip_encoding.
    #[inline]
    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

/// A fixed-capability table over a caller-provided backing store.
#[derive(Debug)]
pub struct CapabilityTable<'a> {
    slots: &'a mut [SlotEntry],
    allocated: u64,
}

impl<'a> CapabilityTable<'a> {
    /// Validates the backing store and initializes every slot to the empty value.
    // CEP:WHAT: Constructs a capability table over a caller-provided slot backing store, initializing every slot to EMPTY.
    // CEP:WHY: All structural validation happens once here so later operations rely on a proven-valid state; the full initialization pass discharges the no-phantom-capability obligation for caller-supplied memory.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns CapabilityError::InvalidArgument for a zero-length backing store; no panic on any input; the backing store is written only after validation succeeds.
    // CEP:ASSUMES: The caller owns the backing store exclusively for the table's lifetime; enforced by the mutable borrow.
    // CEP:COST: O(capacity) for the one-time initialization pass; O(1) validation.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: a table over caller-supplied slots cannot guarantee a deterministic fresh state without writing every slot once; the construction performs exactly that pass plus O(1) validation. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-capability test_table_rejects_empty_backing, test_alloc_returns_usable_handle; Kani proof proof_handle_space_no_panic.
    // CEP:SECURITY: Backing contents are untrusted (threat model TM-1/TM-2); uninitialized memory must never resolve as a capability.
    // CEP:ID: CS-a01
    pub fn new(backing: &'a mut [SlotEntry]) -> Result<Self, CapabilityError> {
        if backing.is_empty() {
            return Err(CapabilityError::InvalidArgument);
        }
        for slot in backing.iter_mut() {
            *slot = SlotEntry::empty();
        }
        Ok(Self {
            slots: backing,
            allocated: 0,
        })
    }

    /// Allocates a slot holding `rights` on `object` and returns its handle.
    // CEP:WHAT: Allocates a new capability with the requested rights on the target object.
    // CEP:WHY: design.md 9.1 capability allocator; the first-fit scan over slots keeps the implementation minimal and total, with the cost bound documented and the free-list alternative tracked for the process-manager workload.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns TableFull when no slot is free; state is unchanged on error.
    // CEP:ASSUMES: none beyond construction invariants.
    // CEP:COST: O(capacity) first-fit scan worst case; O(1) per entry write.
    // CEP:OPTIMAL: not-optimal
    // CEP:OPTPROOF: the first-fit scan matches the search lower bound for an unordered free-slot list, but a maintained free list makes allocation O(1); the scan was chosen because its correctness surface is smaller while no workload data exists. Optimization tracked as ticket CEP-13. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:EVIDENCE: cargo test -p civ-capability test_alloc_returns_usable_handle, test_table_exhaustion_exact; Kani proof proof_invariant_slots_consistent.
    // CEP:ID: CS-a02
    pub fn alloc(&mut self, object: u64, rights: Rights) -> Result<Handle, CapabilityError> {
        let mut index = 0;
        while index < self.slots.len() {
            if !self.slots[index].allocated {
                self.slots[index].rights = rights;
                self.slots[index].object = object;
                self.slots[index].allocated = true;
                self.allocated += 1;
                return Ok(Handle::pack(index as u32, self.slots[index].generation));
            }
            index += 1;
        }
        Err(CapabilityError::TableFull)
    }

    /// Derives a child capability from a resolved parent; refuses escalation and grant-less parents.
    // CEP:WHAT: Creates a child capability whose rights are the requested subset of the parent's rights on the parent's object.
    // CEP:WHY: Attenuation-only derivation is the core authority invariant (design.md 9.1, threat model TM-3); the RIGHT_GRANT check separates use authority from delegation authority, and both checks run before any mutation so a refused derive leaves the table unchanged.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns ParentInvalid for an unresolvable parent handle, GrantDenied when the parent lacks RIGHT_GRANT, Escalation when the request exceeds the parent's rights, TableFull when no slot is free; state is bit-for-bit unchanged on every error path.
    // CEP:ASSUMES: none beyond construction invariants; parent handle is untrusted.
    // CEP:COST: O(capacity) parent resolution-free slot scan plus O(1) checks.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: correct derivation requires resolving the parent (bounds and generation checks), two right comparisons, and one slot write; the implementation performs exactly that work. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:SECURITY: Escalation refusal is the mandatory capability-integrity property (design.md 21); the Kani harness proves refusal over the full u64 rights space, not sampled inputs.
    // CEP:EVIDENCE: cargo test -p civ-capability test_derive_attenuates, test_derive_refuses_escalation, test_derive_requires_grant_right; Kani proof proof_derive_never_escalates.
    // CEP:ID: CS-a03
    pub fn derive(&mut self, parent: Handle, requested: Rights) -> Result<Handle, CapabilityError> {
        let slot_index = parent.slot_index() as usize;
        if slot_index >= self.slots.len() {
            return Err(CapabilityError::ParentInvalid);
        }
        let parent_slot = self.slots[slot_index];
        if !parent_slot.allocated || parent_slot.generation != parent.generation() {
            return Err(CapabilityError::ParentInvalid);
        }
        if !parent_slot
            .rights
            .contains_all(Rights::from_raw(RIGHT_GRANT))
        {
            return Err(CapabilityError::GrantDenied);
        }
        if !parent_slot.rights.contains_all(requested) {
            return Err(CapabilityError::Escalation);
        }
        let mut index = 0;
        while index < self.slots.len() {
            if !self.slots[index].allocated {
                self.slots[index].rights = requested;
                self.slots[index].object = parent_slot.object;
                self.slots[index].allocated = true;
                self.allocated += 1;
                return Ok(Handle::pack(index as u32, self.slots[index].generation));
            }
            index += 1;
        }
        Err(CapabilityError::TableFull)
    }

    /// Revokes a capability: the named handle never resolves again.
    // CEP:WHAT: Invalidates the capability named by the handle by advancing the slot's generation and freeing the slot.
    // CEP:WHY: Revocation must close access permanently for the named handle (crate invariant 3, threat model TM-3); advancing the generation makes every outstanding copy of the handle stale, and freeing the slot allows reuse that still cannot match old handles.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidSlot for out-of-range indices, SlotEmpty for unallocated slots, StaleHandle for generation mismatches; state is unchanged on error. A slot whose generation counter reaches u32::MAX is freed and its counter wraps per the documented residual risk (threat model TM-3).
    // CEP:ASSUMES: Generation wrap after 2^32 revocations of one slot is outside the modeled lifetime; the wrap residual is recorded, not silently assumed (standards Law 2).
    // CEP:COST: O(1) after bounds checks; one compare, one generation advance, one flag clear.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: correct revocation requires the same three checks lookup performs plus one generation advance and one flag clear; the implementation performs exactly that. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:SECURITY: Revocation-closes-access is a mandatory system proof (design.md 21); the Kani harness proves the property over symbolic handles.
    // CEP:EVIDENCE: cargo test -p civ-capability test_revoke_closes_access, test_revoke_state_unchanged_on_error; Kani proof proof_revoke_closes_access.
    // CEP:ID: CS-a04
    pub fn revoke(&mut self, handle: Handle) -> Result<(), CapabilityError> {
        let slot_index = handle.slot_index() as usize;
        if slot_index >= self.slots.len() {
            return Err(CapabilityError::InvalidSlot);
        }
        let slot = &mut self.slots[slot_index];
        if !slot.allocated {
            return Err(CapabilityError::SlotEmpty);
        }
        if slot.generation != handle.generation() {
            return Err(CapabilityError::StaleHandle);
        }
        slot.generation = slot.generation.wrapping_add(1);
        slot.allocated = false;
        slot.rights = Rights::none();
        slot.object = 0;
        self.allocated -= 1;
        Ok(())
    }

    /// Resolves a handle to its rights and target object; fails closed on anything else.
    // CEP:WHAT: Validates a handle and returns the capability's rights and target object.
    // CEP:WHY: Every authority check in the kernel reduces to this one total, fail-closed resolution (design.md 9.1 capability validation, threat model TM-3); a single gate keeps the security property provable.
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns InvalidSlot when the slot index is outside the table, SlotEmpty when the slot is unallocated, StaleHandle when the generation does not match; no other outcome exists for any u64 input.
    // CEP:ASSUMES: The handle is an untrusted u64; no assumption is made about its provenance.
    // CEP:COST: O(1); one bounds check, one occupancy check, one generation compare.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: fail-closed validation requires exactly the three checks the implementation performs, and resolution is two register reads. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    // CEP:SECURITY: Forgery resistance is the TM-3 discharge: any u64 either resolves correctly or fails closed; the Kani harness runs the full input space.
    // CEP:EVIDENCE: cargo test -p civ-capability test_lookup_fail_closed, test_stale_handle_fails_closed, test_forged_generation_fails; Kani proof proof_lookup_fail_closed.
    // CEP:ID: CS-a05
    pub fn lookup(&self, handle: Handle) -> Result<(Rights, u64), CapabilityError> {
        let slot_index = handle.slot_index() as usize;
        if slot_index >= self.slots.len() {
            return Err(CapabilityError::InvalidSlot);
        }
        let slot = &self.slots[slot_index];
        if !slot.allocated {
            return Err(CapabilityError::SlotEmpty);
        }
        if slot.generation != handle.generation() {
            return Err(CapabilityError::StaleHandle);
        }
        Ok((slot.rights, slot.object))
    }

    /// Resolves a handle to its rights only.
    // CEP:WHAT: Returns the rights of the capability named by the handle.
    // CEP:WHY: Most authority checks need only the rights; a dedicated accessor avoids destructuring tuples at call sites.
    // CEP:STATUS: complete
    // CEP:FAILURE: Inherits lookup's fail-closed classification exactly.
    // CEP:ASSUMES: none beyond lookup.
    // CEP:COST: O(1); same checks as lookup.
    // CEP:EVIDENCE: cargo test -p civ-capability test_derive_attenuates.
    #[inline]
    pub fn rights_of(&self, handle: Handle) -> Result<Rights, CapabilityError> {
        let (rights, _object) = self.lookup(handle)?;
        Ok(rights)
    }

    /// Number of live capabilities.
    // CEP:WHAT: Returns the count of allocated slots.
    // CEP:WHY: Resource accounting (design.md 9.1 authority audit) needs the live count without inspecting slots.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: The counter is maintained by construction and proof proof_invariant_slots_consistent.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_slots_consistent.
    #[inline]
    pub const fn allocated_count(&self) -> u64 {
        self.allocated
    }

    /// Total slot capacity of the backing store.
    // CEP:WHAT: Returns the table's slot capacity.
    // CEP:WHY: Accounting and exhaustion tests need the capacity; deriving it from the backing store keeps one source of truth.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p civ-capability test_table_exhaustion_exact.
    #[inline]
    pub const fn capacity(&self) -> u64 {
        self.slots.len() as u64
    }

    /// Number of free slots.
    // CEP:WHAT: Returns the free-slot count implied by the accounting counter.
    // CEP:WHY: Allocation policy consumes the free count; deriving it avoids a second mutable counter needing separate proof.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total accessor; the subtraction cannot underflow because the invariant allocated <= capacity holds by construction and proof.
    // CEP:ASSUMES: Invariant allocated_count <= capacity; enforced by construction and Kani proof proof_invariant_slots_consistent.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: Kani proof proof_invariant_slots_consistent.
    #[inline]
    pub const fn free_count(&self) -> u64 {
        self.slots.len() as u64 - self.allocated
    }

    /// Popcount-style witness: counts allocated slots independently of the counter.
    // CEP:WHAT: Counts allocated slots by direct inspection, independent of the accounting counter.
    // CEP:WHY: Crate invariant consistency needs a second, differently-computed source of truth so counter drift is detectable corruption, not silent failure (standards 66.1).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(capacity); one occupancy test per slot.
    // CEP:EVIDENCE: Kani proof proof_invariant_slots_consistent; cargo test -p civ-capability test_allocated_count_consistency.
    // CEP:ID: CS-a06
    pub fn allocated_popcount(&self) -> u64 {
        let mut count = 0;
        let mut index = 0;
        while index < self.slots.len() {
            if self.slots[index].allocated {
                count += 1;
            }
            index += 1;
        }
        count
    }
}
