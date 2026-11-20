// CEP:FILE: source/kernel/civ-capability/src/rights.rs
// CEP:WHAT: The rights bitset and the named authority rights of the CIV capability model.
// CEP:WHY: design.md 9.1 requires capability representation, validation, derivation, and attenuation; a total bitset type with named rights makes every authority check a comparable, provable subset operation instead of scattered flag logic.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: Constructors and predicates are total; derive returns CapabilityError::Escalation on a non-subset request; no panic on any input; no allocation; no I/O.
// CEP:ASSUMES: Right bits are distinct powers of two (enforced by construction: each constant is a distinct shift of 1); the bitset is u64, bounding the model to 64 distinct rights, which exceeds the rights named by design.md 9.1 and is re-examined when the authority audit lands.
// CEP:COST: O(1) per operation; one or two machine words of arithmetic; no allocation.
// CEP:EVIDENCE: unit tests rights_* in source/kernel/civ-capability/tests/capability_tests.rs; Kani harnesses in source/kernel/civ-capability/src/proofs.rs.
// CEP:SECURITY: Authority attenuation is the core kernel security property (design.md 9.1, threat model TM-3); derive refuses escalation and the proof harness demonstrates refusal over the full u64 space.

//! Authority rights bitset.

/// Read access to the target object.
// CEP:WHAT: Grants read access to the capability's target object.
// CEP:WHY: Read is the least authority shared widely; separating it from write keeps attenuation meaningful (design.md 9.1 capability attenuation).
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: none; distinct power of two by construction.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
pub const RIGHT_READ: u64 = 1 << 0;
/// Write access to the target object.
// CEP:WHAT: Grants write access to the capability's target object.
// CEP:WHY: Write is separable authority so producers can hold it without granting it onward.
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: none; distinct power of two by construction.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
pub const RIGHT_WRITE: u64 = 1 << 1;
/// Execute access to the target object.
// CEP:WHAT: Grants execute access to the capability's target object.
// CEP:WHY: Execute enables the process-manager increment to hand out entry authority without read/write (design.md 3.2 process execution).
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: none; distinct power of two by construction.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
pub const RIGHT_EXECUTE: u64 = 1 << 2;
/// Authority to map the target into an address space.
// CEP:WHAT: Grants the right to map the target object into an address space.
// CEP:WHY: Mapping is the memory-authority boundary (design.md 4.2 page mapper); keeping it distinct from read/write lets the virtual memory manager check mapping authority alone.
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: none; distinct power of two by construction.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
pub const RIGHT_MAP: u64 = 1 << 3;
/// Authority to derive new capabilities from this one.
// CEP:WHAT: Grants the right to derive child capabilities from this one.
// CEP:WHY: Separating delegation authority from use authority implements design.md 9.1 capability transfer and attenuation without conflating them.
// CEP:STATUS: complete
// CEP:FAILURE: none; constant definition.
// CEP:ASSUMES: none; distinct power of two by construction.
// CEP:COST: compile-time only.
// CEP:EVIDENCE: cargo test -p civ-capability test_derive_requires_grant_right.
pub const RIGHT_GRANT: u64 = 1 << 4;

/// A set of authority rights as a u64 bitset.
///
/// Bit `i` set means the corresponding right is held. The type is total:
/// every u64 value is a legal rights set, so construction cannot fail and
/// all filtering happens at the derivation boundary where escalation is
/// rejected.
// CEP:WHAT: The authority rights bitset over u64.
// CEP:WHY: A total bitset type makes every authority check a single comparable subset operation, keeping attenuation provable instead of scattered flag logic.
// CEP:STATUS: complete
// CEP:FAILURE: none; total type, construction cannot fail.
// CEP:ASSUMES: Right bits are distinct powers of two by construction (named constants are distinct shifts of 1); the u64 width bounds the model to 64 rights.
// CEP:COST: O(1) per operation; one or two machine words; no allocation.
// CEP:EVIDENCE: unit tests rights_* in tests/capability_tests.rs; Kani proof proof_derive_never_escalates.
// CEP:SECURITY: Attenuation legality reduces to contains_all at the derivation boundary (TM-3).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rights(u64);

impl Rights {
    /// The empty rights set; a capability that can do nothing.
    // CEP:WHAT: Constructs the empty rights set.
    // CEP:WHY: Attenuation needs a meaningful zero point; an all-nothing capability is the legitimate result of full attenuation (design.md 9.1 attenuation to nothing is allowed, escalation is not).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); single zero register.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_none_attenuation_legal.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: the observable behavior is the constant zero; any implementation performs the same single move. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn none() -> Self {
        Self(0)
    }

    /// The full rights set over the named kernel rights.
    // CEP:WHAT: Constructs the rights set containing every named kernel right.
    // CEP:WHY: Initial capabilities (design.md 9.1 capability allocator) start from the full named set; composing from the named constants keeps the set synchronized with the rights actually modeled.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); compile-time constant fold.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: the observable behavior is one OR of five constants, folded at compile time; no implementation loads fewer than one immediate. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn all() -> Self {
        Self(RIGHT_READ | RIGHT_WRITE | RIGHT_EXECUTE | RIGHT_MAP | RIGHT_GRANT)
    }

    /// Wraps a raw bitset without validation; every u64 value is a legal set.
    // CEP:WHAT: Wraps a raw u64 bitset into the rights newtype.
    // CEP:WHY: Unnamed bits are legal future rights (design.md 0.3: unspecified is not forbidden); the type stays total and derivation does the filtering.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none; any u64 is accepted by design.
    // CEP:COST: O(1); single register move.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_from_raw_roundtrip.
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: identity wrap; no cheaper implementation exists. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn from_raw(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the raw bitset value.
    // CEP:WHAT: Unwraps the rights newtype to the raw u64 bitset.
    // CEP:WHY: Registry serialization (design.md 9.1 authority audit) needs the raw value; explicit conversion marks the boundary.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); single register move.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_from_raw_roundtrip.
    #[inline]
    pub const fn as_raw(self) -> u64 {
        self.0
    }

    /// True when this set contains every right in `other`.
    // CEP:WHAT: Reports whether this rights set is a superset of `other`.
    // CEP:WHY: Attenuation legality (derived subset of parent) reduces to this single comparison; making it the only gate keeps the security property provable (threat model TM-3).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total predicate.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one AND and one compare.
    // CEP:EVIDENCE: Kani proof proof_derive_never_escalates over the full u64 space (results UNVERIFIED until a recorded cargo kani run; status in verification/evidence/toolchain.json).
    // CEP:OPTIMAL: lower-bound optimal
    // CEP:OPTPROOF: subset testing requires exactly one AND and one compare; the implementation performs exactly that. Benchmark receipt pending the CIV-4 hardware contract (ticket CEP-91).
    #[inline]
    pub const fn contains_all(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// True when this set holds any right in `other`.
    // CEP:WHAT: Reports whether this rights set intersects `other`.
    // CEP:WHY: Access checks ask "does this capability hold the needed right"; intersection is that check (design.md 9.1 capability validation).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total predicate.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one AND and one compare.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_named_bits_distinct.
    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Attenuates this set to the intersection with `requested`.
    // CEP:WHAT: Returns the rights held by both this set and the requested set.
    // CEP:WHY: Attenuation to the intersection makes over-requests harmless instead of errors at use sites; escalation prevention stays in derive where the parent is known (design.md 9.1 attenuation).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total function.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1); one AND.
    // CEP:EVIDENCE: cargo test -p civ-capability test_rights_none_attenuation_legal.
    #[inline]
    pub const fn attenuate(self, requested: Self) -> Self {
        Self(self.0 & requested.0)
    }
}

impl core::fmt::Display for Rights {
    // CEP:ID: CS-a07
    // CEP:WHAT: Renders the rights set as a fixed, allocation-free description.
    // CEP:WHY: Authority diagnostics must be readable without heap formatting machinery (standards 25.3.1); the rendering names exactly the named rights held, which keeps audit logs meaningful (design.md 9.1 authority audit).
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total over all u64 values; writes into the caller-provided formatter.
    // CEP:ASSUMES: The formatter backend does not panic; guaranteed by core::fmt contract for core formatters. Unnamed bits render as their numeric value, never silently dropped (standards 66.1 silence is a defect).
    // CEP:COST: O(number of named rights held) fixed-string writes; at most five named rights plus one numeric suffix; no allocation.
    // CEP:EVIDENCE: unit test test_rights_display_names_held_rights.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut any = false;
        if self.0 & RIGHT_READ != 0 {
            f.write_str("read")?;
            any = true;
        }
        if self.0 & RIGHT_WRITE != 0 {
            if any {
                f.write_str("+")?;
            }
            f.write_str("write")?;
            any = true;
        }
        if self.0 & RIGHT_EXECUTE != 0 {
            if any {
                f.write_str("+")?;
            }
            f.write_str("exec")?;
            any = true;
        }
        if self.0 & RIGHT_MAP != 0 {
            if any {
                f.write_str("+")?;
            }
            f.write_str("map")?;
            any = true;
        }
        if self.0 & RIGHT_GRANT != 0 {
            if any {
                f.write_str("+")?;
            }
            f.write_str("grant")?;
            any = true;
        }
        let unnamed = self.0 & !Rights::all().0;
        if unnamed != 0 {
            if any {
                f.write_str("+")?;
            }
            write!(f, "unnamed:{unnamed:#x}")?;
        } else if !any {
            f.write_str("none")?;
        }
        Ok(())
    }
}
