//! `Category::Partition` driver — FLEET TASK (unimplemented stub).
//!
//! A partition isn't run like a whole sort: its `run_with_input` takes a pivot
//! input, partitions the array around it, and emits the scan; its correctness
//! battery checks the partition invariant (everything left of the returned
//! index ≤ pivot ≤ everything right). Model it on the real
//! `crates/registries/quick_partition_registry/src/lib.rs` `run_with_input`.
//!
//! To implement (in your worktree): render the two fn bodies below against
//! `ctx.abi`, add a faithful `Partitioner` ABI shape + a stub partition type in
//! `avb_abi`/`demo`, and add a test that a `Category::Partition` entry drives
//! the capture logger. Touch ONLY this file + your stub + your test.

use super::{DriverCode, DriverCtx};

pub fn driver(_ctx: &DriverCtx) -> Result<DriverCode, String> {
    Err("TODO(fleet): Partition driver — see docs/todo.md Phase 0.4".into())
}
