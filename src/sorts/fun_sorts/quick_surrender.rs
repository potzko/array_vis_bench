//! Quick surrender — selection sort via quickselect.
//!
//! For each prefix slot `i`, run a quickselect over `arr[i..]` asking for
//! the smallest element (target = 0). The element is left at index `i`
//! and the rest of the slice is shuffled but otherwise unsorted —
//! traditional selection sort's `O(N^2)` outer loop, but each find-min
//! step is a recursive partition rather than a linear scan, so the
//! visualiser sees a partition-heavy trace.
//!
//! Parametrised over a [`QuickSelect`] strategy (recursive vs iterative)
//! which itself fans out over [`PartitionScheme`] × [`PivotSelector`].

use std::marker::PhantomData;

use crate::sorts::quick_selects::quick_select::QuickSelect;
use crate::traits::log_traits::SortLogger;

pub struct QuickSurrender<QS: QuickSelect>(PhantomData<QS>);

impl<QS: QuickSelect> QuickSurrender<QS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        for i in 0..n {
            QS::select(&mut arr[i..], logger, 0);
        }
    }
}

combo_codegen::sort_family!(
    type = QuickSurrender<{Alg}<{P}, {V}>>,
    uses = [
        "crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, ThreeWay}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther}",
        "crate::sorts::quick_selects::quick_select::{IterativeQuickSelect, RecursiveQuickSelect}",
        "super::quick_surrender::QuickSurrender",
    ],
    Alg: QuickSelect,
    P: inline [
        ("Lomuto",   "lomuto"),
        ("Hoare",    "hoare"),
        ("ThreeWay", "three-way"),
        ("Block",    "block"),
    ],
    V: PivotSelector,
    name = "quick surrender",
    big_o = "O(N^2)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "quick surrender", "{Alg}", "{P}", "{V}"],
    max_n_for_tests = 500,
);
