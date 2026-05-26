//! Quickselect — partition-based k-th-order-statistic finder.
//!
//! Each impl reorders `arr` so that the element which *would* end up at
//! position `target` after a full sort lands there. Nothing else is
//! guaranteed to be in order; elements before / after the target form
//! two unsorted partitions.
//!
//! Concrete impls are parametrised over a [`PartitionScheme`] (Lomuto,
//! Hoare, ThreeWay, Block, …) and a [`PivotSelector`] (first,
//! median-of-3, median-of-medians, …). Two strategies are provided:
//!
//! - [`RecursiveQuickSelect`] — straightforward recursion into whichever
//!   half contains `target`.
//! - [`IterativeQuickSelect`] — same control flow, but the tail
//!   recursion is unrolled into a loop. Useful when call-stack depth
//!   matters.

use std::marker::PhantomData;

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PivotQuality,
    PivotSelector, QuickSelect,
};
use sort_logger::SortLogger;

// ── RecursiveQuickSelect ─────────────────────────────────────────────────────

pub struct RecursiveQuickSelect<P: PartitionScheme, V: PivotSelector>(PhantomData<(P, V)>);

impl<P: PartitionScheme, V: PivotSelector> QuickSelect for RecursiveQuickSelect<P, V> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        recursive::<T, U, P, V>(arr, logger, target);
    }
}

/// Helper visitor extracting `(left_end, right_start)` from a
/// single-pivot PartitionScheme call. Records the end of the first
/// unsorted range as `left_end` and the start of the second as
/// `right_start`; everything in `[left_end, right_start)` is placed.
struct BoundsVisitor { left_end: usize, right_start: usize, n: u8 }
impl BoundsVisitor {
    #[inline(always)]
    fn new(len: usize) -> Self { Self { left_end: 0, right_start: len, n: 0 } }
}
impl array_vis_bench_traits::PartitionVisitor for BoundsVisitor {
    #[inline(always)]
    fn unsorted(&mut self, r: std::ops::Range<usize>) {
        if self.n == 0 {
            self.left_end = r.end;
        } else if self.n == 1 {
            self.right_start = r.start;
        }
        self.n += 1;
    }
}

fn recursive<T, U, P, V>(arr: &mut [T], logger: &mut U, target: usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
{
    if arr.len() < 2 {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let mut v = BoundsVisitor::new(arr.len());
    P::partition(arr, logger, &[pivot_idx], &mut v);
    if target < v.left_end {
        recursive::<T, U, P, V>(&mut arr[..v.left_end], logger, target);
    } else if target >= v.right_start {
        recursive::<T, U, P, V>(&mut arr[v.right_start..], logger, target - v.right_start);
    }
    // else: target sits in [left_end, right_start) — already placed.
}

// ── IterativeQuickSelect ─────────────────────────────────────────────────────

pub struct IterativeQuickSelect<P: PartitionScheme, V: PivotSelector>(PhantomData<(P, V)>);

impl<P: PartitionScheme, V: PivotSelector> QuickSelect for IterativeQuickSelect<P, V> {
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
            let pivot_idx = V::select(slice, logger);
            let mut v = BoundsVisitor::new(slice.len());
            P::partition(slice, logger, &[pivot_idx], &mut v);
            if target < v.left_end {
                hi = lo + v.left_end;
            } else if target >= v.right_start {
                lo += v.right_start;
                target -= v.right_start;
            } else {
                return;
            }
        }
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// QuickSelect is the one-sided cousin of QuickSort: each level does
// partition + pivot work, then recurses into a single half. Expected
// depth is O(1) levels with good pivots (each cuts the input by half),
// or O(N) if pivots can degenerate (e.g. first-element on sorted input).

macro_rules! impl_qs_annotations {
    ($ty:ident, $space:expr) => {
        impl<P, V> HasTimeBounds for $ty<P, V>
        where
            P: PartitionScheme + HasTimeBounds,
            V: PivotSelector + HasTimeBounds + PivotQuality,
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
            V: PivotSelector + HasSpace,
        {
            const SPACE: Complexity = Complexity::sum(
                $space,
                Complexity::sum(P::SPACE, V::SPACE),
            );
        }
        impl<P, V> HasStability for $ty<P, V>
        where
            P: PartitionScheme,
            V: PivotSelector,
        {
            /// Quickselect leaves both sides unsorted, so the surrounding
            /// stability question is moot — the algorithm offers no
            /// guarantee about equal-key order.
            const STABLE: bool = false;
        }
    };
}

impl_qs_annotations!(RecursiveQuickSelect, Complexity::LOG_N);
impl_qs_annotations!(IterativeQuickSelect, Complexity::CONST);
