//! The operations on a forest of trees laid out in post order, whose roots
//! ascend from the first to the last so that the last element outranks every
//! other. `rectify` restores the order of the roots after the last one changed,
//! `add` adds an element at the end, `remove_last` takes the last element off and sets its two
//! subtrees among the roots.

use core::marker::PhantomData;

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::shape::{Forest, Shape};
use crate::sift::Sift;
use crate::tree::{above, children, swap};

pub struct SmoothHeap<D, S, F>(PhantomData<(D, S, F)>);

impl<D: Direction, S: Shape, F: Sift> SmoothHeap<D, S, F> {
    /// Puts the root of the last tree among the roots before it. A root before
    /// it that outranks it trades places with it. The last tree is a heap, so
    /// its children rank below the root that went in, but a tree after that has
    /// children of the root that left it, and the root that comes in has to
    /// outrank those too or it stays where it is. The tree that took the
    /// smaller element is sifted once, where the chain of trades stopped.
    /// `orders` ends with the order of the last tree, rooted at `root`.
    pub fn rectify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], orders: &[usize], mut root: usize, log: &mut U) {
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

    /// Adds the element at `at`, the next cell after the forest: it becomes a
    /// tree of its own or the root over the last two trees, sifted into place.
    /// The roots are left as they were, for `rectify` to put in order.
    pub fn add<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, at: usize, log: &mut U) {
        let order = forest.grow::<S>();
        F::sift::<D, S, T, U>(arr, at, order, log);
    }

    /// Takes the last element off a forest of `len` elements. It is the root of
    /// the last tree, which falls apart into its two subtrees, each already a
    /// heap, and each put among the roots in turn.
    pub fn remove_last<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, len: usize, log: &mut U) {
        let order = forest.pop();
        let Some(pair) = children::<S>(len - 1, order) else { return };
        for (child, child_order) in pair {
            forest.push(child_order);
            Self::rectify(arr, forest.orders(), child, log);
        }
    }
}
