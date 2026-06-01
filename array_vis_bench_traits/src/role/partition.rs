//! `PartitionScheme` — partition algorithm role.
//!
//! Implemented by leaf crates like `partition_lomuto` so they can stay
//! tiny and reference only this trait crate + `sort_logger`. The wiring
//! crate (`array_vis_bench_full`) consumes the trait via the partition
//! family's `[[package.metadata.array_vis_bench.families]]` cross-product.
//!
//! Each `partition` call announces its unsorted regions through a
//! [`PartitionVisitor`]; sorted regions (the pivots themselves, any
//! equal-pinned runs) are the implicit gaps between successive
//! `unsorted` calls. The trait generalises over pivot arity via
//! [`PartitionScheme::N_PIVOTS`] so single-pivot impls (LeftLeftPartition, LeftRightPartition,
//! ThreeWay, MovingPivot, Block, …) and dual-pivot impls
//! (DualPivotPartition) share one surface.

use core::ops::Range;
use sort_logger::SortLogger;

/// Allocate `P`'s partition scratch buffer once, run `body` with it, then
/// free it — the single place that owns a quicksort/quickselect scratch
/// buffer for the whole run. Mirrors how the merge sorts wrap their work
/// between one `create_aux_arr_t` / `free_aux_arr_t` pair. Schemes that
/// need no scratch ([`PartitionScheme::SCRATCH_LEN`] `== 0`) skip the
/// aux-array events entirely and `body` receives an empty slice.
///
/// Drivers thread the `&mut [usize]` slice `body` is given into every
/// recursive partition call so the buffer is reused instead of
/// reallocated per call.
#[inline]
pub fn with_partition_scratch<P, T, U, R>(
    logger: &mut U,
    body: impl FnOnce(&mut U, &mut [usize]) -> R,
) -> R
where
    P: PartitionScheme,
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    if P::SCRATCH_LEN == 0 {
        body(logger, &mut [])
    } else {
        let mut scratch = logger.create_aux_arr(P::SCRATCH_LEN);
        let out = body(logger, &mut scratch);
        logger.free_aux_arr(&scratch);
        out
    }
}

/// Receives the unsorted regions a partition leaves behind. The
/// quicksort driver implements this to schedule each region for
/// recursive sorting; sorted regions are the gaps between calls.
pub trait PartitionVisitor {
    /// A region of the array that still needs sorting.
    fn unsorted(&mut self, range: Range<usize>);
}

/// One partition step. Implementors take an array and a slice of
/// pre-selected pivot indices, do the actual swapping work, and
/// announce each unsorted region they leave behind through `visitor`.
///
/// Generalised over pivot arity via [`Self::N_PIVOTS`] so the same
/// trait covers single-pivot partitions (`N_PIVOTS = 1`) and
/// dual-pivot partitions (`N_PIVOTS = 2`) uniformly.
pub trait PartitionScheme {
    /// Display name used in the registry's menu path.
    const NAME: &'static str;
    /// Pivot arity. The caller supplies a `pivots` slice of exactly
    /// this length per call.
    const N_PIVOTS: usize;
    /// Length of the reusable `usize` scratch buffer this scheme needs
    /// per partition call. The quicksort driver allocates a buffer of
    /// exactly this length **once** at the start of the whole sort and
    /// hands the same slice to every `partition` call (mirroring how the
    /// merge sorts pre-allocate their auxiliary array once via
    /// `RotationMerge::scratch_size`). Most in-place schemes need none —
    /// the default `0` makes the driver pass an empty slice. Block
    /// partition overrides this to size its two offset buffers.
    const SCRATCH_LEN: usize = 0;
    /// Partition `arr` using the pivots originally at the indices in
    /// `pivots` (length == [`Self::N_PIVOTS`]). Each unsorted region
    /// is announced to `visitor`; placed regions are the implicit
    /// gaps between calls and don't need to be reported.
    ///
    /// `scratch` is a reusable `usize` buffer of length
    /// [`Self::SCRATCH_LEN`], owned and logged once by the driver for the
    /// lifetime of the sort — schemes must treat its contents as
    /// uninitialised on entry and not assume anything survives between
    /// calls. Schemes with `SCRATCH_LEN == 0` receive an empty slice.
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        scratch: &mut [usize],
        visitor: &mut V,
    )
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor;
}
