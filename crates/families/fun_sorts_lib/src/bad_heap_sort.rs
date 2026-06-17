//! "Bad heap" sort — recursive variant.
//!
//! Treats the array as a malformed binary heap where the children of
//! position `i` are at `i + 1` (left) and `i * 2 + 1` (right). The left
//! child is always the next slot, regardless of depth — so the tree is
//! degenerate and the recursion fans out unpredictably. After each
//! conditional swap, the affected subtree is re-entered.

use sort_logger::SortLogger;

pub struct BadHeapSort;

impl BadHeapSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        sort_rec(arr, 0, arr.len(), logger);
    }
}

fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 2 {
        return;
    }
    let left = start + 1;
    let right = start * 2 + 1;
    if right < end && logger.cond_swap_lt(arr, right, left) {
        sort_rec(arr, right, end, logger);
    }
    sort_rec(arr, left, end, logger);
    if logger.cond_swap_lt(arr, left, start) {
        sort_rec(arr, start, end, logger);
    }
}

#[cfg(feature = "self_register")]
sort_registry_macro::sort_family! {
    type Sort = BadHeapSort;
    name        = "bad heap sort";
    big_o       = "O(N^?)";
    stable      = false;
    direct_sort = true;
    path        = ["fun sorts", "bad heap sort"];
    // The asymmetric "heap" causes the recursion to fan out
    // unpredictably; observed >5s wall-clock well before n=1000.
    max_n_for_tests = 200;
}
