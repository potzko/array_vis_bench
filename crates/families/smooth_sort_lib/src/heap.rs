//! The operations on a forest of trees laid out in post order, whose roots
//! ascend from the first to the last so that the last element outranks every
//! other, or whose roots are left as they come. `settle` puts the last root in
//! its place among the roots, `add` adds an element at the end, `heapify_tree` makes one whole tree a
//! heap, and `remove_last` takes the last element off and sets its two
//! subtrees among the roots.

use core::marker::PhantomData;

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::roots::Roots;
use crate::shape::{Forest, Shape};
use crate::sift::Sift;
use crate::tree::children;

pub struct SmoothHeap<D, S, F, R>(PhantomData<(D, S, F, R)>);

impl<D: Direction, S: Shape, F: Sift, R: Roots> SmoothHeap<D, S, F, R> {
    /// Puts the root of the last tree in its place among the roots, as `R`
    /// keeps them.
    #[inline(always)]
    pub fn settle<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], orders: &[usize], root: usize, log: &mut U) {
        R::settle::<D, S, F, T, U>(arr, orders, root, log);
    }

    /// Adds the element at `at`, the next cell after the forest: it becomes a
    /// tree of its own or the root over the last two trees, sifted into place.
    /// The roots are left as they were, for `settle` to deal with.
    pub fn add<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, at: usize, log: &mut U) {
        let order = forest.grow::<S>();
        F::sift::<D, S, T, U>(arr, at, order, log);
    }

    /// Makes the tree of `order` rooted at `root` a heap, its subtrees first.
    pub fn heapify_tree<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], root: usize, order: usize, log: &mut U) {
        let Some(pair) = children::<S>(root, order) else { return };
        for (child, child_order) in pair { Self::heapify_tree(arr, child, child_order, log); }
        F::sift::<D, S, T, U>(arr, root, order, log);
    }

    /// Takes the last element off a forest of `len` elements. It is the root of
    /// the last tree, which falls apart into its two subtrees, each already a
    /// heap, and each put among the roots in turn.
    pub fn remove_last<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, len: usize, log: &mut U) {
        let order = forest.pop();
        let Some(pair) = children::<S>(len - 1, order) else { return };
        for (child, child_order) in pair {
            forest.push(child_order);
            Self::settle(arr, forest.orders(), child, log);
        }
    }
}
