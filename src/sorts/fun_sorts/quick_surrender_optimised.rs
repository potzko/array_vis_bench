//! Quick surrender (optimised) — block quickselect plus small sort.
//!
//! Sweep the array left-to-right in fixed-size blocks of `SS::THRESHOLD`.
//! Each block: run `QS::select(arr[i..], block - 1)` so the block's slot
//! ends up holding the `block` smallest elements of the remaining tail
//! (unordered), then have `SS` sort that block in place. The block size
//! comes from the [`NonTrivialSmallSort`]'s threshold, so it's always ≥ 2.

use std::marker::PhantomData;

use crate::sorts::quick_selects::quick_select::QuickSelect;
use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::NonTrivialSmallSort;

pub struct QuickSurrenderOptimised<QS: QuickSelect, SS: NonTrivialSmallSort>(
    PhantomData<(QS, SS)>,
);

impl<QS: QuickSelect, SS: NonTrivialSmallSort> QuickSurrenderOptimised<QS, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        let block_size = SS::THRESHOLD;
        let mut i = 0;
        while i < n {
            let block = (n - i).min(block_size);
            QS::select(&mut arr[i..], logger, block - 1);
            SS::sort(&mut arr[i..i + block], logger);
            i += block;
        }
    }
}

combo_codegen::sort_family!(
    type = QuickSurrenderOptimised<{Alg}<{P}, {V}>, {SS}>,
    uses = [
        "crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, ThreeWay}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther}",
        "crate::sorts::quick_selects::quick_select::{IterativeQuickSelect, RecursiveQuickSelect}",
        "crate::utils::small_sort::{InsertionSmallSort, Network16SmallSort, NetworkSmallSort}",
        "crate::utils::small_sort::Size2SmallSort",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::quick_surrender_optimised::QuickSurrenderOptimised",
    ],
    Alg: QuickSelect,
    P: inline [
        ("Lomuto",   "lomuto"),
        ("Hoare",    "hoare"),
        ("ThreeWay", "three-way"),
        ("Block",    "block"),
    ],
    V: PivotSelector,
    SS: NonTrivialSmallSort,
    name = "quick surrender optimised",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "quick surrender optimised", "{Alg}", "{P}", "{V}", "{SS}"],
    max_n_for_tests = 1000,
);
