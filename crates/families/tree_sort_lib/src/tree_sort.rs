//! `TreeSort<B>`, the driver of the tree sort family, and `Tree`, the role
//! `B` fills. Every element is attached as a leaf of a binary search tree
//! over the packed node store, then the leftmost node is deleted n times and
//! written at the next output slot, so the sorted prefix grows from the left
//! the way heap sort's sorted suffix grows from the right. `B` decides what
//! happens to the shape after each attach and each deletion.

use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::{Layout, Nodes, Packed};

pub trait Tree {
    // The rank `attach` writes into a new leaf: the height 1 for a height
    // balanced tree, 0 for a tree that never reads the field, so its words
    // hold nothing but links.
    const LEAF_RANK: usize;

    // Node `node` was just attached as a leaf.
    fn inserted<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, node: usize);

    // The leftmost node was just unlinked from `parent`, which is `nil` when
    // it was the root.
    fn removed<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, parent: usize);
}

pub struct TreeSort<B: Tree>(PhantomData<B>);

impl<B: Tree> TreeSort<B> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        assert!(n <= Packed::REACH, "tree sort takes at most {} elements, got {n}", Packed::REACH);
        let mut nodes = Nodes::<T, U, Packed>::new(arr, logger);
        for i in 0..n {
            nodes.attach(i, B::LEAF_RANK);
            B::inserted(&mut nodes, i);
        }
        for k in 0..n {
            let (least, parent) = nodes.unlink_min();
            B::removed(&mut nodes, parent);
            nodes.place(least, k);
        }
        nodes.free();
    }
}

array_vis_bench_traits::direct_sort!(TreeSort<B>; B: Tree);

impl<B: Tree + HasTimeBounds> HasTimeBounds for TreeSort<B> {
    const WORST: Complexity = B::WORST;
    const BEST: Complexity = B::BEST;
    const AVERAGE: Complexity = B::AVERAGE;
}

impl<B: Tree> HasSpace for TreeSort<B> {
    const SPACE: Complexity = Complexity::N1;
}

// An equal key goes right on attach, so the in order sequence is sorted by
// key then by index, and rotations, leftmost deletion and relocation all keep
// that sequence whatever the tree does with the shape.
impl<B: Tree> HasStability for TreeSort<B> {
    const STABLE: bool = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_input::pseudo_random;
    use crate::{Avl, Naive};
    use sort_logger::NoOpLogger;

    fn assert_sorts<B: Tree>(mut arr: Vec<usize>) {
        let mut want = arr.clone();
        want.sort();
        TreeSort::<B>::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr, want);
    }

    fn assert_sorts_everything<B: Tree>() {
        for n in 0..200 { assert_sorts::<B>(pseudo_random(n, n / 3 + 1)); }
        assert_sorts::<B>((0..500).collect());
        assert_sorts::<B>((0..500).rev().collect());
        assert_sorts::<B>(vec![7; 300]);
        assert_sorts::<B>(pseudo_random(5000, 1 << 20));
    }

    #[test]
    fn the_avl_tree_sorts() { assert_sorts_everything::<Avl>(); }

    #[test]
    fn the_naive_tree_sorts() { assert_sorts_everything::<Naive>(); }

    // Ordered by key alone, so the index records where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Keyed { key: u8, index: usize }
    impl PartialEq for Keyed { fn eq(&self, other: &Self) -> bool { self.key == other.key } }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
    }
    impl Ord for Keyed {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.key.cmp(&other.key) }
    }

    fn assert_stable<B: Tree>() {
        let keys = pseudo_random(400, 5);
        let mut arr: Vec<Keyed> =
            keys.iter().enumerate().map(|(index, &key)| Keyed { key: key as u8, index }).collect();
        TreeSort::<B>::sort(&mut arr, &mut NoOpLogger);
        let in_order = |w: &[Keyed]| w[0].key < w[1].key || (w[0].key == w[1].key && w[0].index < w[1].index);
        assert!(arr.windows(2).all(in_order), "{arr:?}");
    }

    #[test]
    fn both_trees_keep_equal_keys_in_input_order() {
        assert_stable::<Avl>();
        assert_stable::<Naive>();
    }

    #[test]
    fn annotations_come_from_the_tree() {
        assert_eq!(<TreeSort<Avl> as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<TreeSort<Naive> as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<TreeSort<Naive> as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert!(!<TreeSort<Avl> as HasSpace>::SPACE.is_in_place());
    }
}
