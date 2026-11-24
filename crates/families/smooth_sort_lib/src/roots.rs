//! What a heap does about the order of its roots. `SortedRoots` keeps them
//! ascending, each put in place as it joins, so that the greatest element is
//! always last and an extraction costs nothing to find it. `ScannedRoots` leaves
//! them as they come and finds the greatest at each extraction, a comparison for
//! each root, as the poplar sort does.

use array_vis_bench_traits::Complexity;
use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::shape::{Forest, Shape};
use crate::sift::Sift;
use crate::tree::{above, children, swap};

pub trait Roots {
    /// Whether the roots ascend, the last outranking the rest.
    const ORDERED: bool;

    /// The best case of a sort that keeps its roots this way.
    const BEST: Complexity;

    /// Puts the root of the last tree in its place among the roots before it.
    /// `orders` ends with the order of the last tree, rooted at `root`.
    fn settle<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], orders: &[usize], root: usize, log: &mut U);

    /// Brings the greatest element of a forest of `len` elements to the last
    /// cell, ahead of the removal of the last element.
    fn raise<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &Forest, len: usize, log: &mut U);
}

/// The roots ascending. A root before the last that outranks it trades places
/// with it. The last tree is a heap, so its children rank below the root that
/// went in, but a tree after that has children of the root that left it, and the
/// root that comes in has to outrank those too or it stays where it is. The tree
/// that took the smaller element is sifted once, where the chain of trades
/// stopped. The greatest is last already, and nothing is raised.
pub struct SortedRoots;

impl Roots for SortedRoots {
    const ORDERED: bool = true;
    const BEST: Complexity = Complexity::N1;

    fn settle<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], orders: &[usize], mut root: usize, log: &mut U) {
        let last = orders.len() - 1;
        let mut at = last;
        while at > 0 {
            let before = root - S::size(orders[at]);
            if !above::<D, T, U>(arr, before, root, log) { break; }
            let held_down = at < last && children::<S>(root, orders[at]).is_some_and(|pair| pair.iter().any(|&(child, _)| !above::<D, T, U>(arr, before, child, log)));
            if held_down { break; }
            swap::<D, T, U>(arr, before, root, log);
            root = before;
            at -= 1;
        }
        if at < last { F::sift::<D, S, T, U>(arr, root, orders[at], log); }
    }

    fn raise<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _forest: &Forest, _len: usize, _log: &mut U) {}
}

/// The roots as they come. Every extraction compares the roots from the last
/// back for the greatest, one comparison each, trades it with the last element,
/// and sifts the tree that took the last element's place. Nothing is ordered
/// when an element joins, so a sorted input costs a scan an element.
pub struct ScannedRoots;

impl Roots for ScannedRoots {
    const ORDERED: bool = false;
    const BEST: Complexity = Complexity::N_LOG_N;

    fn settle<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _orders: &[usize], _root: usize, _log: &mut U) {}

    fn raise<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &Forest, len: usize, log: &mut U) {
        let orders = forest.orders();
        let last = orders.len() - 1;
        let (mut root, mut at) = (len - 1, last);
        // The greatest so far, as its root and its tree; a tie keeps the later.
        let (mut greatest, mut tree) = (root, last);
        while at > 0 {
            root -= S::size(orders[at]);
            at -= 1;
            if above::<D, T, U>(arr, root, greatest, log) { (greatest, tree) = (root, at); }
        }
        if tree < last {
            swap::<D, T, U>(arr, greatest, len - 1, log);
            F::sift::<D, S, T, U>(arr, greatest, orders[tree], log);
        }
    }
}
