//! Slow sort — the "multiply and surrender" anti-sort.
//!
//! Recursively sort each half, surface the larger of the two midpoints to
//! the array's end via a conditional swap, then recurse on everything
//! *except* the now-placed last element. Worst case ~`O(N^logN)`, which
//! is the whole point of the algorithm — no fallback, just slow.

use sort_logger::SortLogger;

pub struct SlowSort;

impl SlowSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        slow_sort_rec::<T, U>(arr, logger);
    }
}

fn slow_sort_rec<T, U>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    let len = arr.len();
    if len < 2 {
        return;
    }
    let mid = len / 2;
    slow_sort_rec::<T, U>(&mut arr[..mid], logger);
    slow_sort_rec::<T, U>(&mut arr[mid..], logger);
    logger.cond_swap_gt(arr, mid - 1, len - 1);
    slow_sort_rec::<T, U>(&mut arr[..len - 1], logger);
}

sort_registry_macro::sort_family! {
    type Sort = SlowSort;
    name        = "slow sort";
    big_o       = "O(N^logN)";
    stable      = false;
    direct_sort = true;
    path        = ["fun sorts", "slow sort"];
    max_n_for_tests = 150;
}
