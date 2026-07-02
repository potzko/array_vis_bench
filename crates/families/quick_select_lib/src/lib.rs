//! Quickselect — partition-based k-th-order-statistic finder.
//!
//! Each impl reorders `arr` so that the element which *would* end up at
//! position `target` after a full sort lands there. Nothing else is
//! guaranteed to be in order; elements before / after the target form
//! unsorted partitions.
//!
//! Concrete impls are parametrised over ONE self-pivoting [`PartitionScheme`]
//! (`N_PIVOTS = 0`) — a `Pivoted<Scheme, Sel>` (a pivot-consuming scheme fused
//! with its selector) or `HeapExtract` (genuinely pivotless). The partition owns
//! pivot selection; the driver supplies none. This is the SAME single-partition
//! contract `CanonicalQuickSort` uses, so quick + quick-select share one partition
//! role-system. Two strategies are provided:
//!
//! - [`RecursiveQuickSelect`] — straightforward recursion into whichever
//!   region contains `target`.
//! - [`IterativeQuickSelect`] — same control flow, but the tail
//!   recursion is unrolled into a loop. Useful when call-stack depth
//!   matters.
//!
//! This is the one-sided cousin of `CanonicalQuickSort<P, SS>`: where quicksort
//! recurses into *every* unsorted region the partition emits, quickselect
//! recurses into the *single* region containing `target` and drops the rest.
//! Dual-pivot quickselect is just `RecursiveQuickSelect<Pivoted<DualPivotPartition,
//! DPS>>`.

use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::{
    with_partition_scratch, Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme,
    PartitionVisitor, PivotQuality, QuickSelect,
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

pub struct RecursiveQuickSelect<P: PartitionScheme>(PhantomData<P>);

impl<P: PartitionScheme> QuickSelect for RecursiveQuickSelect<P> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            recursive::<T, U, P>(arr, scratch, logger, target);
        });
    }
}

fn recursive<T, U, P>(arr: &mut [T], scratch: &mut [usize], logger: &mut U, target: usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
{
    if arr.len() < 2 {
        return;
    }
    let mut v = RegionVisitor::new();
    // Self-pivoting partition (`N_PIVOTS == 0`): it picks its own pivots
    // internally, so the driver supplies none.
    P::partition::<T, U, _>(arr, logger, &[], scratch, &mut v);
    if let Some(r) = region_for(&v, target) {
        let (start, end) = (r.start, r.end);
        recursive::<T, U, P>(&mut arr[start..end], scratch, logger, target - start);
    }
}

// ── IterativeQuickSelect ─────────────────────────────────────────────────────

pub struct IterativeQuickSelect<P: PartitionScheme>(PhantomData<P>);

impl<P: PartitionScheme> QuickSelect for IterativeQuickSelect<P> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            let mut lo = 0usize;
            let mut hi = arr.len();
            let mut target = target;
            while hi - lo >= 2 {
                let slice = &mut arr[lo..hi];
                let mut v = RegionVisitor::new();
                P::partition::<T, U, _>(slice, logger, &[], scratch, &mut v);
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
        });
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// QuickSelect is the one-sided cousin of QuickSort: each level does the
// partition's work (pivot pick + split, both now INSIDE the self-pivoting
// partition `P`), then recurses into a single region. Expected depth is O(1)
// levels with good pivots, or O(N) if the partition's pivot can degenerate
// (`P::DEGENERATES`). The bounds read off `P` alone — a `Pivoted<Scheme, Sel>`
// or `HeapExtract` already fuses scheme + selector one level down.

macro_rules! impl_qs_annotations {
    ($ty:ident, $space:expr) => {
        impl<P> HasTimeBounds for $ty<P>
        where
            P: PartitionScheme + HasTimeBounds + PivotQuality,
        {
            const WORST: Complexity = Complexity::product(
                if P::DEGENERATES { Complexity::N1 } else { Complexity::CONST },
                P::WORST,
            );
            const BEST: Complexity = P::BEST;
            const AVERAGE: Complexity = P::AVERAGE;
        }
        impl<P> HasSpace for $ty<P>
        where
            P: PartitionScheme + HasSpace,
        {
            const SPACE: Complexity = Complexity::sum($space, P::SPACE);
        }
        impl<P> HasStability for $ty<P>
        where
            P: PartitionScheme,
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

/// A distinct type-head alias for [`RecursiveQuickSelect`], used ONLY as the inner
/// select of the recursive set-deep-heapify family. It exists so that family can
/// reference its select without competing for the `RecursiveQuickSelect` type-head
/// (owned by the quick-select family). As an alias it is the SAME type, so it sorts
/// identically and inherits every impl.
pub type SetSelect<P> = RecursiveQuickSelect<P>;
