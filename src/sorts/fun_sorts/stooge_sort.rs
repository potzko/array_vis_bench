//! Stooge sort — recursive three-way swap-and-recurse.
//!
//! On each call: swap the first and last elements into order, then if
//! `n > 2` recurse on the first two-thirds, the last two-thirds, and the
//! first two-thirds again. Famous as a slow comparison sort —
//! `O(N^{log 3 / log 1.5}) ≈ O(N^2.71)`.
//!
//! Parametrised over [`SmallSort`] so the recursion's bottom can switch
//! to a sane sort once subarrays are small; the cap on random-input
//! sizes keeps worst-case n manageable in tests.

use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::SmallSort;

pub struct StoogeSort<SS: SmallSort>(PhantomData<SS>);

impl<SS: SmallSort> StoogeSort<SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        stooge_sort_rec::<T, U, SS>(arr, logger);
    }
}

fn stooge_sort_rec<T, U, SS>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    SS: SmallSort,
{
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    let n = arr.len();
    if n < 2 {
        return;
    }
    logger.cond_swap_gt(arr, 0, n - 1);
    if n > 2 {
        let third = n / 3;
        stooge_sort_rec::<T, U, SS>(&mut arr[..n - third], logger);
        stooge_sort_rec::<T, U, SS>(&mut arr[third..], logger);
        stooge_sort_rec::<T, U, SS>(&mut arr[..n - third], logger);
    }
}

combo_codegen::sort_family!(
    type = StoogeSort<{SS}>,
    uses = [
        "crate::utils::small_sort::{InsertionSmallSort, Network16SmallSort, NetworkSmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::stooge_sort::StoogeSort",
    ],
    SS: inline [
        ("InsertionSmallSort<LinearInsertion, 16>",  "insertion: 16"),
        ("InsertionSmallSort<LinearInsertion, 32>",  "insertion: 32"),
        ("InsertionSmallSort<BinaryInsertion, 16>",  "binary insertion: 16"),
        ("InsertionSmallSort<BinaryInsertion, 32>",  "binary insertion: 32"),
        ("NetworkSmallSort",                          "network: 8"),
        ("Network16SmallSort",                        "network: 16"),
    ],
    name = "stooge sort",
    big_o = "O(N^2.71)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "stooge sort", "{SS}"],
    max_n_for_tests = 500,
);
