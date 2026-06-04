//! `Category::Rotation` driver — FLEET TASK (unimplemented stub).
//!
//! A rotation takes an array + a split point and rotates the two halves,
//! emitting the moves; its correctness battery checks the result equals the
//! expected cyclic shift. Model it on the real `ROTATION_INPUTS` shape
//! (`generate -> (Vec<usize>, mid)`).
//!
//! Implement the two fn bodies against `ctx.abi`, add the faithful rotation ABI
//! shape + a stub rotation type, and a test. Touch ONLY this file + stub + test.

use super::{DriverCode, DriverCtx};

pub fn driver(_ctx: &DriverCtx) -> Result<DriverCode, String> {
    Err("TODO(fleet): Rotation driver — see docs/todo.md Phase 0.4".into())
}
