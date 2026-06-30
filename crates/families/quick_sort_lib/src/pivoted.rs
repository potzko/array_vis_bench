//! `Pivoted<Scheme, Sel>` — a self-pivoting partition.
//!
//! Wraps a pivot-consuming [`PartitionScheme`] (`Scheme`, e.g.
//! `LeftLeftPartition`) together with a [`PivotInput`] (`Sel`, e.g.
//! `FirstElement` or a dual `CombinedSelector`) into a partition that
//! **picks its own pivots** and therefore needs none supplied from
//! outside. It itself impls [`PartitionScheme`] with `N_PIVOTS = 0`, so
//! it drops straight into the pivot-free quicksort driver
//! ([`CanonicalQuickSort`](crate::quick_sort::CanonicalQuickSort)).
//!
//! This is what moves pivot ownership *off* the sort and *into* the
//! partition: instead of `QuickSort<Scheme, Sel, SS>` (sort declares the
//! pivot), the composition is `CanonicalQuickSort<Pivoted<Scheme, Sel>,
//! SS>` (partition declares the pivot). The arity coupling
//! `Sel::N == Scheme::N_PIVOTS` is internal to this wrapper.

use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
use array_vis_bench_traits::{
    Complexity, PartitionScheme, PartitionVisitor, PivotInput,
};
use sort_logger::SortLogger;

/// A `Scheme` + `Sel` fused into a zero-external-pivot partition.
pub struct Pivoted<Scheme, Sel>(PhantomData<(Scheme, Sel)>);

impl<Scheme: PartitionScheme, Sel: PivotInput> PartitionScheme for Pivoted<Scheme, Sel> {
    const NAME: &'static str = Scheme::NAME;
    /// Self-pivoting: the driver supplies no pivots.
    const N_PIVOTS: usize = 0;
    /// Same scratch the inner scheme needs — the driver allocates it once
    /// and threads it straight through.
    const SCRATCH_LEN: usize = Scheme::SCRATCH_LEN;

    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        _pivots: &[usize],
        scratch: &mut [usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        // Stack buffer sized for the largest supported arity (2); only
        // `Sel::N` slots are populated after monomorphisation.
        let mut pivots = [0usize; 2];
        Sel::pick(arr, logger, &mut pivots);
        Scheme::partition::<T, U, V>(arr, logger, &pivots[..Sel::N], scratch, visitor);
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// A pivoted partition's cost is its scheme's partition work plus its
// selector's pick work; its space is the max of the two; it is stable
// only if both are; and it degenerates iff its pivot selector can
// degenerate — exactly the quantities `QuickSort<Scheme, Sel, SS>` used
// to read from `Scheme` and `Sel` separately, now combined one level
// down so the driver reads them off the partition alone.

impl<Scheme, Sel> HasTimeBounds for Pivoted<Scheme, Sel>
where
    Scheme: PartitionScheme + HasTimeBounds,
    Sel: PivotInput + HasTimeBounds,
{
    const WORST: Complexity = Complexity::sum(Scheme::WORST, Sel::WORST);
    const BEST: Complexity = Complexity::sum(Scheme::BEST, Sel::BEST);
    const AVERAGE: Complexity = Complexity::sum(Scheme::AVERAGE, Sel::AVERAGE);
}

impl<Scheme, Sel> HasSpace for Pivoted<Scheme, Sel>
where
    Scheme: PartitionScheme + HasSpace,
    Sel: PivotInput + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(Scheme::SPACE, Sel::SPACE);
}

impl<Scheme, Sel> HasStability for Pivoted<Scheme, Sel>
where
    Scheme: PartitionScheme + HasStability,
    Sel: PivotInput + HasStability,
{
    const STABLE: bool = Scheme::STABLE && Sel::STABLE;
}

impl<Scheme, Sel> PivotQuality for Pivoted<Scheme, Sel>
where
    Scheme: PartitionScheme,
    Sel: PivotInput + PivotQuality,
{
    const DEGENERATES: bool = Sel::DEGENERATES;
}
