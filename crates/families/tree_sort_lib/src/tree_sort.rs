//! `TreeSort<B>`, the driver of the tree sort family, and `Tree`, the role
//! `B` fills. Every element is attached as a leaf of a binary search tree
//! over the node store, then the leftmost node is deleted n times and
//! written at the next output slot, so the sorted prefix grows from the left
//! the way heap sort's sorted suffix grows from the right. `B` decides what
//! happens to the shape after each attach and each deletion.

use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::{Halves, Layout, Nodes, Packed, Whole};

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
    // The shortest layout that reaches the array: a node takes one word up
    // to 2^19 elements, two words up to 2^32, then a word for each field.
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n <= Packed::REACH { Self::sort_in::<T, U, Packed>(arr, logger) }
        else if n <= Halves::REACH { Self::sort_in::<T, U, Halves>(arr, logger) }
        else { Self::sort_in::<T, U, Whole>(arr, logger) }
    }

    pub fn sort_in<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let mut meta = logger.create_aux_arr(L::words(n));
        let mut nodes = Nodes::<T, U, L>::over(arr, &mut meta, logger);
        for i in 0..n {
            nodes.attach(i, B::LEAF_RANK);
            B::inserted(&mut nodes, i);
        }
        for k in 0..n {
            let (least, parent) = nodes.unlink_min();
            B::removed(&mut nodes, parent);
            nodes.place(least, k);
        }
        logger.free_aux_arr(&meta);
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
    use crate::test_input::{each_slack, pseudo_random};
    use crate::{Avl, FixedAvl, Naive};
    use sort_logger::NoOpLogger;

    fn assert_sorted_by(mut arr: Vec<usize>, sort: impl Fn(&mut [usize])) {
        let mut want = arr.clone();
        want.sort();
        sort(&mut arr);
        assert_eq!(arr, want);
    }

    fn assert_sorts<B: Tree, L: Layout>(arr: Vec<usize>) {
        assert_sorted_by(arr, |arr| TreeSort::<B>::sort_in::<usize, NoOpLogger, L>(arr, &mut NoOpLogger));
    }

    fn assert_sorts_everything<B: Tree, L: Layout>() {
        for n in 0..200 { assert_sorts::<B, L>(pseudo_random(n, n / 3 + 1)); }
        assert_sorts::<B, L>((0..500).collect());
        assert_sorts::<B, L>((0..500).rev().collect());
        assert_sorts::<B, L>(vec![7; 300]);
        assert_sorts::<B, L>(pseudo_random(5000, 1 << 20));
    }

    fn assert_sorts_in_every_layout<B: Tree>() {
        assert_sorts_everything::<B, Packed>();
        assert_sorts_everything::<B, Halves>();
        assert_sorts_everything::<B, Whole>();
    }

    fn assert_avl_sorts<const SLACK: usize>() { assert_sorts_in_every_layout::<Avl<SLACK>>(); }

    #[test]
    fn the_avl_tree_sorts_at_every_slack() { each_slack!(assert_avl_sorts); }

    #[test]
    fn the_fixed_avl_tree_sorts() { assert_sorts_in_every_layout::<FixedAvl>(); }

    #[test]
    fn the_naive_tree_sorts() { assert_sorts_in_every_layout::<Naive>(); }

    // Either side of 2^19 elements `sort` takes the one word layout or the next.
    #[test]
    fn sort_takes_the_layout_that_reaches_the_array() {
        for n in [Packed::REACH, Packed::REACH + 1, 1_000_000] {
            assert_sorted_by(pseudo_random(n, 1 << 30), |arr| TreeSort::<Avl<1>>::sort(arr, &mut NoOpLogger));
            assert_sorted_by(pseudo_random(n, 1 << 30), |arr| TreeSort::<Avl<32>>::sort(arr, &mut NoOpLogger));
            assert_sorted_by(pseudo_random(n, 1 << 30), |arr| TreeSort::<FixedAvl>::sort(arr, &mut NoOpLogger));
            assert_sorted_by(pseudo_random(n, 1 << 30), |arr| TreeSort::<Naive>::sort(arr, &mut NoOpLogger));
        }
    }

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

    fn assert_stable<B: Tree, L: Layout>() {
        let keys = pseudo_random(400, 5);
        let mut arr: Vec<Keyed> =
            keys.iter().enumerate().map(|(index, &key)| Keyed { key: key as u8, index }).collect();
        TreeSort::<B>::sort_in::<Keyed, NoOpLogger, L>(&mut arr, &mut NoOpLogger);
        let in_order = |w: &[Keyed]| w[0].key < w[1].key || (w[0].key == w[1].key && w[0].index < w[1].index);
        assert!(arr.windows(2).all(in_order), "{arr:?}");
    }

    fn assert_avl_stable<const SLACK: usize>() {
        assert_stable::<Avl<SLACK>, Packed>();
        assert_stable::<Avl<SLACK>, Halves>();
        assert_stable::<Avl<SLACK>, Whole>();
    }

    #[test]
    fn every_tree_keeps_equal_keys_in_input_order_in_every_layout() {
        each_slack!(assert_avl_stable);
        assert_stable::<FixedAvl, Packed>();
        assert_stable::<FixedAvl, Halves>();
        assert_stable::<FixedAvl, Whole>();
        assert_stable::<Naive, Packed>();
        assert_stable::<Naive, Halves>();
        assert_stable::<Naive, Whole>();
    }

    fn assert_avl_is_n_log_n<const SLACK: usize>() {
        let bounds = [<TreeSort<Avl<SLACK>> as HasTimeBounds>::WORST, <TreeSort<Avl<SLACK>> as HasTimeBounds>::BEST];
        assert_eq!(bounds, [Complexity::N_LOG_N; 2]);
        assert_eq!(<TreeSort<Avl<SLACK>> as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
    }

    #[test]
    fn annotations_come_from_the_tree() {
        each_slack!(assert_avl_is_n_log_n);
        assert_eq!(<TreeSort<FixedAvl> as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<TreeSort<Naive> as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<TreeSort<Naive> as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert!(!<TreeSort<Avl<1>> as HasSpace>::SPACE.is_in_place());
    }
}
