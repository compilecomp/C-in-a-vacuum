// CEP:FILE: source/kernel/civ-pmm/src/lib.rs
// CEP:WHAT: Crate root for the CIV physical memory manager (design.md 4.1).
// CEP:WHY: Wires frame identity, the free-frame bitmap, and the frame allocator behind one no_std surface so kernel callers depend on a single verified unit with a single error type.
// CEP:CLASS: CEP-1
// CEP:STATUS: complete
// CEP:FAILURE: none; module wiring only, no runtime failure paths of its own.
// CEP:ASSUMES: One allocator instance per bitmap backing store at a time; enforced by the borrow checker through the &mut [u64] backing store.
// CEP:COST: compile-time wiring only; no runtime instructions beyond the referenced modules.
// CEP:EVIDENCE: cargo test -p civ-pmm (33 scenarios); cargo kani -p civ-pmm (8 harnesses); entries in verification/proof-registry.json.
// CEP:SECURITY: All caller inputs are treated as untrusted (security/threat_model.md TM-1, TM-2); every API is total and returns typed errors.
//! CIV physical memory manager.
//!
//! A bitmap-based physical frame allocator over a caller-provided backing
//! store. The allocator owns no memory of its own and never allocates;
//! the backing store is a mutable slice of bitmap words supplied at
//! construction (standards 25.5 allocation policy).
//!
//! # Invariants
//!
//! 1. Bitmap bit `i` reflects exactly the allocation state of frame
//!    `base_frame + i`; bits at or beyond `frame_count` are always zero.
//! 2. `allocated_count` equals the popcount of the bitmap at every
//!    observable program point; `allocated_count + free_count` equals
//!    `frame_count`.
//! 3. A frame is either free (bit zero) or owned by exactly one
//!    allocation; freeing a frame whose bit is zero is rejected with
//!    [`PmmError::NotAllocated`] and leaves state unchanged.
//! 4. Every public function is panic-free over the full range of its
//!    inputs; all arithmetic is checked and all index computations are
//!    bounds-checked before use.
//!
//! # Concurrency
//!
//! The allocator is single-owner and not `Sync`/`Send` beyond the backing
//! store's autotraits. Per-CPU allocation caches (design.md 4.1) are a
//! later increment and will define their own ordering contract; this
//! module makes no atomicity promises of its own.
#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod allocator;
pub mod bitmap;
pub mod error;
pub mod frame;

#[cfg(kani)]
mod proofs;

pub use allocator::FrameAllocator;
pub use bitmap::Bitmap;
pub use error::PmmError;
pub use frame::{Frame, FrameRange, PhysAddr, FRAME_SIZE_BYTES};
