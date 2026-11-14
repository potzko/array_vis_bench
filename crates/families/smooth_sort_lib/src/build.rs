//! How the forest is first made: `Insert` grows it one element at a time, each
//! element rectified into the roots as it comes, `LazyInsert` leaves out the
//! rectify of a tree the next element is about to join, and `Heapify` lays the
//! whole forest down first and makes each tree a heap from the leaves up.

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::heap::SmoothHeap;
use crate::shape::{Forest, Shape};
use crate::sift::Sift;

pub trait Build {
    /// Makes a forest of the whole array, `forest` empty on the way in.
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U);
}

// A tree the next element joins is a child by then, and where a child stands
// among the roots is of no account, so the lazy build leaves it unrectified.
fn insert_all<const LAZY: bool, D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
    let n = arr.len();
    for at in 0..n {
        SmoothHeap::<D, S, F>::add(arr, forest, at, log);
        let joined = LAZY && at + 1 < n && forest.merges_next::<S>();
        if !joined { SmoothHeap::<D, S, F>::rectify(arr, forest.orders(), at, log); }
    }
}

/// Every element rectified as it is added.
pub struct Insert;

impl Build for Insert {
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
        insert_all::<false, D, S, F, T, U>(arr, forest, log);
    }
}

/// Every element rectified as it is added, but the last of a tree that the next
/// element joins.
pub struct LazyInsert;

impl Build for LazyInsert {
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
        insert_all::<true, D, S, F, T, U>(arr, forest, log);
    }
}

/// The forest laid down for the whole length, each tree made a heap, and then
/// each root rectified among those before it, as an insertion sort of the roots.
pub struct Heapify;

impl Build for Heapify {
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
        (0..arr.len()).for_each(|_| { forest.grow::<S>(); });
        let orders = forest.orders();
        let ends = orders.iter().scan(0, |end, &order| { *end += S::size(order); Some(*end - 1) });
        for (tree, root) in ends.enumerate() {
            SmoothHeap::<D, S, F>::heapify_tree(arr, root, orders[tree], log);
            SmoothHeap::<D, S, F>::rectify(arr, &orders[..=tree], root, log);
        }
    }
}
