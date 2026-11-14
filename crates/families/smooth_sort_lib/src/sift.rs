//! The sift of a tree whose subtrees are heaps and whose root may be out of
//! place: `Pairs` takes the greater child at each level and asks whether it
//! outranks the root, two comparisons a level.

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::shape::Shape;
use crate::tree::{above, children, swap};

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
