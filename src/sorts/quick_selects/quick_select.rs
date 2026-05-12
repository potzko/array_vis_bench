//! Quickselect — partition-based k-th-order-statistic finder.
//!
//! Each impl reorders `arr` so that the element which *would* end up at
//! position `target` after a full sort lands there. Nothing else is
//! guaranteed to be in order; elements before / after the target form two
//! unsorted partitions.
//!
//! Concrete impls are parametrised over a [`PartitionScheme`] (Lomuto,
//! Hoare, ThreeWay, Block, …) and a [`PivotSelector`] (first, median-of-3,
//! median-of-medians, …). Two strategies are provided:
//!
//! - [`RecursiveQuickSelect`] — straightforward recursion into whichever
//!   half contains `target`.
//! - [`IterativeQuickSelect`] — same control flow, but the tail recursion
//!   is unrolled into a loop. Useful when call-stack depth matters.

use std::marker::PhantomData;

use crate::sorts::quick_sorts::partitions::PartitionScheme;
use crate::sorts::quick_sorts::pivot_selectors::PivotSelector;
use crate::traits::log_traits::SortLogger;

/// Reorder `arr` so the element that would sit at index `target` after a
/// full sort ends up there. The two surrounding partitions remain
/// unordered.
pub trait QuickSelect {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    );
}

// ── RecursiveQuickSelect ─────────────────────────────────────────────────────

pub struct RecursiveQuickSelect<P: PartitionScheme, V: PivotSelector>(PhantomData<(P, V)>);
combo_codegen::component!(QuickSelect, RecursiveQuickSelect, "recursive");

impl<P: PartitionScheme, V: PivotSelector> QuickSelect for RecursiveQuickSelect<P, V> {
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
    V: PivotSelector,
{
    if arr.len() < 2 {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot_idx);
    if target < left_end {
        recursive::<T, U, P, V>(&mut arr[..left_end], logger, target);
    } else if target >= right_start {
        recursive::<T, U, P, V>(&mut arr[right_start..], logger, target - right_start);
    }
    // else: target sits in [left_end, right_start) — already placed.
}

// ── IterativeQuickSelect ─────────────────────────────────────────────────────

pub struct IterativeQuickSelect<P: PartitionScheme, V: PivotSelector>(PhantomData<(P, V)>);
combo_codegen::component!(QuickSelect, IterativeQuickSelect, "iterative");

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
            let (left_end, right_start) = P::partition(slice, logger, pivot_idx);
            if target < left_end {
                hi = lo + left_end;
            } else if target >= right_start {
                lo += right_start;
                target -= right_start;
            } else {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorts::quick_sorts::partitions::{Hoare, Lomuto, ThreeWay};
    use crate::sorts::quick_sorts::pivot_selectors::{
        FirstElement, LastElement, MedianOfThree, MiddleElement,
    };
    use crate::traits::log_traits::NoOpLogger;

    fn check_placed<QS: QuickSelect>(input: Vec<usize>) {
        let mut sorted = input.clone();
        sorted.sort();
        let logger: &mut NoOpLogger = &mut NoOpLogger;
        for target in 0..input.len() {
            let mut arr = input.clone();
            QS::select(&mut arr, logger, target);
            assert_eq!(
                arr[target], sorted[target],
                "target={target} input={input:?} got={arr:?}"
            );
            for &v in &arr[..target] {
                assert!(v <= sorted[target], "left partition not ≤ target");
            }
            for &v in &arr[target + 1..] {
                assert!(v >= sorted[target], "right partition not ≥ target");
            }
        }
    }

    #[test]
    fn recursive_lomuto_first() {
        check_placed::<RecursiveQuickSelect<Lomuto, FirstElement>>(vec![5, 3, 8, 1, 9, 2, 7, 4, 6]);
    }

    #[test]
    fn iterative_hoare_median() {
        check_placed::<IterativeQuickSelect<Hoare, MedianOfThree>>(vec![5, 3, 8, 1, 9, 2, 7, 4, 6]);
    }

    #[test]
    fn duplicates_three_way() {
        check_placed::<RecursiveQuickSelect<ThreeWay, MiddleElement>>(vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]);
    }

    #[test]
    fn iterative_lomuto_last() {
        check_placed::<IterativeQuickSelect<Lomuto, LastElement>>(vec![7, 7, 7, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn small_arrays() {
        check_placed::<RecursiveQuickSelect<Lomuto, FirstElement>>(vec![]);
        check_placed::<RecursiveQuickSelect<Lomuto, FirstElement>>(vec![42]);
        check_placed::<RecursiveQuickSelect<Lomuto, FirstElement>>(vec![2, 1]);
        check_placed::<IterativeQuickSelect<Hoare, MedianOfThree>>(vec![]);
        check_placed::<IterativeQuickSelect<Hoare, MedianOfThree>>(vec![42]);
        check_placed::<IterativeQuickSelect<Hoare, MedianOfThree>>(vec![2, 1]);
    }
}
