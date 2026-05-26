//! Shaker recursive circle sort.
//!
//! Alternates between pre-order (circle_pass → left → right) and post-order
//! (left → right → circle_pass) at each depth level.  Because the ordering
//! changes with depth — sub-calls are always of the opposite kind — this
//! cannot be expressed as `CircleSortRecursive<Order>` with a single order.

use sort_logger::SortLogger;
use super::orderings::circle_pass;

pub struct CircleSortShakerRecursive;

impl CircleSortShakerRecursive {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        while sort_pre(arr, 0, arr.len() - 1, logger) {}
    }
}

/// Pre-order level: `circle_pass` first, then recurse sub-ranges post-order.
fn sort_pre<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    if start == end {
        return false;
    }
    let pass = circle_pass(arr, start, end, logger);
    let mid = start + (end - start) / 2;
    let left = sort_post(arr, start, mid, logger);
    let right = sort_post(arr, mid + 1, end, logger);
    pass || left || right
}

/// Post-order level: recurse sub-ranges pre-order, then `circle_pass`.
fn sort_post<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    if start == end {
        return false;
    }
    let mid = start + (end - start) / 2;
    let left = sort_pre(arr, start, mid, logger);
    let right = sort_pre(arr, mid + 1, end, logger);
    let pass = circle_pass(arr, start, end, logger);
    pass || left || right
}
