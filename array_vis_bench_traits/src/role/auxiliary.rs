//! `Auxiliary` — the scratchspace a merge uses for its working buffer.
//!
//! This is the composition axis behind block-merge sorts (WikiSort / GrailSort):
//! *how much* auxiliary storage a merge is handed, and therefore whether it
//! copies a run out into a buffer or works in place. It mirrors [`Rotation`]'s
//! "small auxiliary" convention — `scratch_size(n)` cells are allocated once via
//! the logger (`create_aux_arr_t`) and threaded into every merge; `0` means the
//! merge runs fully in place. All element movement goes through the logger.
//!
//! A consumer (`scratchspace_lib::AuxiliaryMerge<A>`) turns an `Auxiliary` into a
//! [`RotationMerge`](crate::role::merge::RotationMerge)
//! so it drops straight into the existing merge pipeline, bench, and visualiser.
//!
//! [`Rotation`]: crate::role::Rotation

use crate::Complexity;

pub trait Auxiliary {
    /// Short label used in the generated variant name (`auxiliary<full>`).
    const NAME: &'static str;

    /// Worst-case time of a merge backed by this auxiliary: `O(n)` when the
    /// buffer can hold a run, `O(n log n)` for the in-place (`scratch_size == 0`)
    /// rotation fallback.
    const MERGE_WORST: Complexity;

    /// Extra space the auxiliary reserves.
    const MERGE_SPACE: Complexity;

    /// Scratch cells to reserve for merging an `n`-element array. `0` = in-place.
    /// The caller allocates this once (via the logger) and threads it into every
    /// merge, exactly like [`Rotation::scratch_size`](crate::role::Rotation::scratch_size).
    fn scratch_size(n: usize) -> usize;

    /// Cells to carve out of the array itself for an `n`-element array: a buffer
    /// of distinct elements that the merge swaps its working values through, so
    /// the array stays whole. `0` for an external or absent buffer. The carved
    /// cells are not part of `scratch_size`, nothing is allocated for them.
    fn internal_size(_n: usize) -> usize {
        0
    }
}
