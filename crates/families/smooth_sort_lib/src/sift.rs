//! The sift of a tree whose subtrees are heaps and whose root may be out of
//! place: `Pairs` takes the greater child at each level and asks whether it
//! outranks the root, two comparisons a level, and `BottomUp` takes the greater
//! child all the way to a leaf and then climbs to where the root belongs, one
//! comparison a level on the way down and a few on the way back, and
//! `BinarySearch` finds that place by a search of the path instead.

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::shape::Shape;
use crate::tree::{above, children, put, swap, take};

pub trait Sift {
    /// Sinks the element at `root`, the root of a tree of `order`, until the
    /// tree is a heap again.
    fn sift<D: Direction, S: Shape, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], root: usize, order: usize, log: &mut U);
}

// The greater of two subtrees, the left one on a tie.
#[inline(always)]
fn greater<D: Direction, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], [left, right]: [(usize, usize); 2], log: &mut U) -> (usize, usize) {
    if above::<D, T, U>(arr, right.0, left.0, log) { right } else { left }
}

/// The classic sift: the greater child, then the root against it.
pub struct Pairs;

impl Sift for Pairs {
    fn sift<D: Direction, S: Shape, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mut root: usize, mut order: usize, log: &mut U) {
        while let Some(pair) = children::<S>(root, order) {
            let (child, child_order) = greater::<D, T, U>(arr, pair, log);
            if !above::<D, T, U>(arr, child, root, log) { break; }
            swap::<D, T, U>(arr, root, child, log);
            (root, order) = (child, child_order);
        }
    }
}

// The deepest path a tree has, an order's worth of cells: a tree holds at most
// a word's count of elements, and the Narayana trees are the deepest, 116 orders.
pub(crate) const DEPTH: usize = 128;

// Walks from the root of the tree of `order` to a leaf by the greater child, one
// comparison a level, leaving the cells passed in `path` and giving how many.
fn descend<D: Direction, S: Shape, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], root: usize, order: usize, path: &mut [usize; DEPTH], log: &mut U) -> usize {
    let mut depth = 0;
    let (mut at, mut at_order) = (root, order);
    while let Some(pair) = children::<S>(at, at_order) {
        (at, at_order) = greater::<D, T, U>(arr, pair, log);
        path[depth] = at;
        depth += 1;
    }
    depth
}

// Shifts the cells of `path` up one place, from the first, into the root and
// each other, and drops the root's element into the last. One write a level.
fn lift<D: Direction, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], root: usize, path: &[usize], log: &mut U) {
    let value = take::<D, T>(arr, root);
    let mut hole = root;
    for &cell in path {
        let up = take::<D, T>(arr, cell);
        put::<D, T, U>(arr, hole, up, log);
        hole = cell;
    }
    if hole != root { put::<D, T, U>(arr, hole, value, log); }
}

/// Floyd's sift, a descent to a leaf by the greater child with one comparison a
/// level, then a climb from the leaf to the lowest cell the root's element does
/// not outrank, where the cells above shift up and the element lands. Moves are
/// one write a level rather than a swap.
pub struct BottomUp;

impl Sift for BottomUp {
    fn sift<D: Direction, S: Shape, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], root: usize, order: usize, log: &mut U) {
        if S::children(order).is_none() { return; }
        let mut path = [0; DEPTH];
        let mut depth = descend::<D, S, T, U>(arr, root, order, &mut path, log);
        while depth > 0 && !above::<D, T, U>(arr, path[depth - 1], root, log) { depth -= 1; }
        lift::<D, T, U>(arr, root, &path[..depth], log);
    }
}

/// The bottom up sift with the climb a binary search. The cells of the path
/// descend, each the greater child of the one before, so the ones the root's
/// element does not outrank are a suffix, and a search finds where they begin in
/// a logarithm of the depth in comparisons. The moves are the same.
pub struct BinarySearch;

impl Sift for BinarySearch {
    fn sift<D: Direction, S: Shape, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], root: usize, order: usize, log: &mut U) {
        if S::children(order).is_none() { return; }
        let mut path = [0; DEPTH];
        let depth = descend::<D, S, T, U>(arr, root, order, &mut path, log);
        // The cells before `low` outrank the element, those from `high` do not.
        let (mut low, mut high) = (0, depth);
        while low < high {
            let mid = (low + high + 1) / 2;
            if above::<D, T, U>(arr, path[mid - 1], root, log) { low = mid } else { high = mid - 1 }
        }
        lift::<D, T, U>(arr, root, &path[..low], log);
    }
}
