//! Quickselect — partition-based k-th-order-statistic finder.
//!
//! Each impl reorders `arr` so that the element which *would* end up at
//! position `target` after a full sort lands there. Nothing else is
//! guaranteed to be in order; elements before / after the target form
//! unsorted partitions.
//!
//! Concrete impls are parametrised over a [`PartitionScheme`] (LeftLeftPartition,
//! LeftRightPartition, ThreeWay, Block, DualPivotPartition, …) and a [`PivotInput`] (a
//! single [`PivotSelector`] for `N_PIVOTS = 1`, or a dual-pivot selector
//! like `CombinedSelector` / `NintherDualPivot` for `N_PIVOTS = 2`). The
//! two arities must agree: `P::N_PIVOTS == V::N`. Two strategies are
//! provided:
//!
//! - [`RecursiveQuickSelect`] — straightforward recursion into whichever
//!   region contains `target`.
//! - [`IterativeQuickSelect`] — same control flow, but the tail
//!   recursion is unrolled into a loop. Useful when call-stack depth
//!   matters.
//!
//! This is the one-sided cousin of `QuickSort<P, V, SS>`: where quicksort
//! recurses into *every* unsorted region the partition emits, quickselect
//! recurses into the *single* region containing `target` and drops the
//! rest. Dual-pivot quickselect is just `QuickSelect<DualPivotPartition, DPS>`
//! — the old standalone `RecursiveDualPivotQuickSelect` /
//! `IterativeDualPivotQuickSelect` types are gone.

use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
    PivotInput, PivotQuality, QuickSelect,
};
use sort_logger::SortLogger;

/// Stack-resident visitor collecting up to 4 unsorted ranges per
/// partition call (3 for DualPivotPartition dual-pivot, 2 for single-pivot).
/// The partition emits them in ascending position order; quickselect
/// relies on that ordering to locate the region holding `target`.
struct RegionVisitor {
    ranges: [Range<usize>; 4],
    n: u8,
}

impl RegionVisitor {
    #[inline(always)]
    fn new() -> Self {
        Self { ranges: [0..0, 0..0, 0..0, 0..0], n: 0 }
    }
}

impl PartitionVisitor for RegionVisitor {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        // Safety: trait contract caps partition emit at 4 ranges per call.
        unsafe { *self.ranges.get_unchecked_mut(self.n as usize) = r };
        self.n += 1;
    }
}

/// Locate the unsorted region (emitted in ascending order) that contains
/// `target` and return it. `None` means `target` landed on a placed
/// element (a gap between regions, or past the last region) and is
/// already in its final position — recursion stops.
#[inline(always)]
fn region_for(v: &RegionVisitor, target: usize) -> Option<Range<usize>> {
    let mut i = 0usize;
    while i < v.n as usize {
        let r = v.ranges[i].clone();
        if target < r.start {
            // Sits in the placed gap before this region.
            return None;
        }
        if target < r.end {
            return Some(r);
        }
        i += 1;
    }
    None
}

// ── RecursiveQuickSelect ─────────────────────────────────────────────────────

pub struct RecursiveQuickSelect<P: PartitionScheme, V: PivotInput>(PhantomData<(P, V)>);

impl<P: PartitionScheme, V: PivotInput> QuickSelect for RecursiveQuickSelect<P, V> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        recursive::<T, U, P, V>(arr, logger, target);
    }
}

fn recursive<T, U, P, V>(arr: &mut [T], logger: &mut U, target: usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotInput,
{
    if arr.len() < 2 {
        return;
    }
    let mut pivots = [0usize; 2];
    V::pick(arr, logger, &mut pivots);
    let mut v = RegionVisitor::new();
    P::partition::<T, U, _>(arr, logger, &pivots[..V::N], &mut v);
    if let Some(r) = region_for(&v, target) {
        let (start, end) = (r.start, r.end);
        recursive::<T, U, P, V>(&mut arr[start..end], logger, target - start);
    }
}

// ── IterativeQuickSelect ─────────────────────────────────────────────────────

pub struct IterativeQuickSelect<P: PartitionScheme, V: PivotInput>(PhantomData<(P, V)>);

impl<P: PartitionScheme, V: PivotInput> QuickSelect for IterativeQuickSelect<P, V> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        let mut lo = 0usize;
        let mut hi = arr.len();
        let mut target = target;
        while hi - lo >= 2 {
            let slice = &mut arr[lo..hi];
            let mut pivots = [0usize; 2];
            V::pick(slice, logger, &mut pivots);
            let mut v = RegionVisitor::new();
            P::partition::<T, U, _>(slice, logger, &pivots[..V::N], &mut v);
            match region_for(&v, target) {
                Some(r) => {
                    // Narrow the window to the chosen region and rebase
                    // target into its local coordinates.
                    hi = lo + r.end;
                    lo += r.start;
                    target -= r.start;
                }
                None => return,
            }
        }
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// QuickSelect is the one-sided cousin of QuickSort: each level does
// partition + pivot work, then recurses into a single region. Expected
// depth is O(1) levels with good pivots (each cuts the input by a
// constant factor), or O(N) if the pivot can degenerate (e.g.
// first-element on sorted input). The bounds mirror
// `QuickSort<P, V, SS>` minus the small-sort slot, with O(1) expected
// depth instead of O(log N) because only one side is followed.

macro_rules! impl_qs_annotations {
    ($ty:ident, $space:expr) => {
        impl<P, V> HasTimeBounds for $ty<P, V>
        where
            P: PartitionScheme + HasTimeBounds,
            V: PivotInput + HasTimeBounds + PivotQuality,
        {
            const WORST: Complexity = Complexity::product(
                if V::DEGENERATES { Complexity::N1 } else { Complexity::CONST },
                Complexity::sum(P::WORST, V::WORST),
            );
            const BEST: Complexity = Complexity::sum(P::BEST, V::BEST);
            const AVERAGE: Complexity = Complexity::sum(P::AVERAGE, V::AVERAGE);
        }
        impl<P, V> HasSpace for $ty<P, V>
        where
            P: PartitionScheme + HasSpace,
            V: PivotInput + HasSpace,
        {
            const SPACE: Complexity = Complexity::sum(
                $space,
                Complexity::sum(P::SPACE, V::SPACE),
            );
        }
        impl<P, V> HasStability for $ty<P, V>
        where
            P: PartitionScheme,
            V: PivotInput,
        {
            /// Quickselect leaves regions unsorted, so the surrounding
            /// stability question is moot — the algorithm offers no
            /// guarantee about equal-key order.
            const STABLE: bool = false;
        }
    };
}

impl_qs_annotations!(RecursiveQuickSelect, Complexity::LOG_N);
impl_qs_annotations!(IterativeQuickSelect, Complexity::CONST);
