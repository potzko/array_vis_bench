//! "Bad heap" sort — heap-extract variant.
//!
//! Build a malformed "heap" then repeatedly swap the root past the end
//! and re-heapify. The malformation: the first `right` child is computed
//! as `ind * 2` (parent-of-zero collapses to zero), but subsequent
//! traversal uses `ind + 2`. The inconsistency is intentional — the tree
//! is broken, so the extract loop only roughly sorts and the result is
//! not actually ordered after one pass.

use sort_logger::SortLogger;

pub struct BadHeapSortAlt;

impl BadHeapSortAlt {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        first_heapify(arr, logger);
        for i in (1..arr.len()).rev() {
            logger.swap(arr, 0, i);
            heapify(arr, 0, i, logger);
        }
    }
}

fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left = ind + 1;
    let mut right = ind * 2;

    if right < end && logger.cmp_gt(arr, right, left) {
        left = right;
    }

    while left < end && logger.cmp_gt(arr, left, ind) {
        logger.swap(arr, ind, left);
        ind = left;
        left = ind + 1;
        right = ind + 2;
        if right < end && logger.cmp_gt(arr, right, left) {
            left = right;
        }
    }
}

fn first_heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for start in (0..arr.len()).rev() {
        heapify(arr, start, arr.len(), logger);
    }
}

sort_registry_macro::sort_family! {
    type Sort = BadHeapSortAlt;
    name        = "bad heap sort alt";
    big_o       = "O(N^2)";
    stable      = false;
    direct_sort = true;
    path        = ["fun sorts", "bad heap sort alt"];
}
