//! `Category::Merge` driver — FLEET TASK (unimplemented stub).
//!
//! A standalone merge takes TWO already-sorted runs and merges them, emitting
//! the merge; its correctness battery checks the merged output is sorted and a
//! permutation of the inputs. Model it on the real `MERGE_INPUTS` shape
//! (`generate -> (Vec<usize>, split_point)`) and `merge_standalone_registry`.
//!
//! Implement the two fn bodies against `ctx.abi`, add the faithful merge ABI
//! shape + a stub merge type, and a test. Touch ONLY this file + stub + test.

use super::{DriverCode, DriverCtx};

pub fn driver(_ctx: &DriverCtx) -> Result<DriverCode, String> {
    Err("TODO(fleet): Merge driver — see docs/todo.md Phase 0.4".into())
}
