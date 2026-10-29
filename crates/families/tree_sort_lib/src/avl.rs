//! `Avl`, the height balanced tree of the tree sort family. After each attach
//! and each leftmost deletion it retraces towards the root, refreshing
//! heights and rotating wherever two siblings differ by more than one level.
//! The rank word of the node store holds the height.

use array_vis_bench_traits::composable::HasTimeBounds;
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::{Layout, Nodes, LEFT, RIGHT};
use crate::tree_sort::Tree;

pub struct Avl;

impl Tree for Avl {
    const LEAF_RANK: usize = 1;

    fn inserted<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, node: usize) {
        retrace(nodes, nodes.parent(node));
    }

    fn removed<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, parent: usize) {
        retrace(nodes, parent);
    }
}

impl HasTimeBounds for Avl {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

// Walk up from `x` restoring heights and balance. Stops at the first node
// whose subtree kept its height, because the heights above it are then
// untouched: an insertion stops at its first rotation, a deletion may rotate
// at several levels.
fn retrace<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, mut x: usize) {
    while x != nodes.nil {
        let parent = nodes.parent(x);
        let before = nodes.rank(x);
        let (left, right) = (nodes.rank(nodes.link(x, LEFT)), nodes.rank(nodes.link(x, RIGHT)));
        let after = if left.abs_diff(right) > 1 {
            let top = rebalance(nodes, x);
            nodes.rank(top)
        } else {
            let after = 1 + left.max(right);
            if after != before { nodes.set_rank(x, after); }
            after
        };
        if after == before { return; }
        x = parent;
    }
}

// Restore balance at `x` with one rotation towards its lighter side, after
// first rotating the heavy child when its inner grandchild is the taller, the
// double rotation case.
fn rebalance<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, x: usize) -> usize {
    let left_heavy = nodes.rank(nodes.link(x, LEFT)) > nodes.rank(nodes.link(x, RIGHT));
    let (side, other) = if left_heavy { (RIGHT, LEFT) } else { (LEFT, RIGHT) };
    let heavy = nodes.link(x, other);
    if nodes.rank(nodes.link(heavy, side)) > nodes.rank(nodes.link(heavy, other)) {
        rotate(nodes, heavy, other, side);
    }
    rotate(nodes, x, side, other)
}

// A rotation followed by the two height refreshes it owes, lower node first.
fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(
    nodes: &mut Nodes<'_, T, U, L>,
    x: usize,
    side: usize,
    other: usize,
) -> usize {
    let y = nodes.rotate(x, side, other);
    refresh_height(nodes, x);
    refresh_height(nodes, y);
    y
}

fn refresh_height<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, i: usize) {
    let height = 1 + nodes.rank(nodes.link(i, LEFT)).max(nodes.rank(nodes.link(i, RIGHT)));
    nodes.set_rank(i, height);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{Halves, Packed, Whole};
    use crate::test_input::pseudo_random;
    use sort_logger::NoOpLogger;

    // The stored height of every node is right, its children differ by at
    // most one level and each child points back at it, so the tree really is
    // an AVL tree and the bound holds. Returns the height.
    fn assert_avl<T: Ord + Copy, L: Layout>(nodes: &Nodes<T, NoOpLogger, L>, i: usize) -> usize {
        if i == nodes.nil { return 0; }
        let (left, right) = (nodes.link(i, LEFT), nodes.link(i, RIGHT));
        let (left_height, right_height) = (assert_avl(nodes, left), assert_avl(nodes, right));
        assert!(left_height.abs_diff(right_height) <= 1, "node {i} is unbalanced");
        assert_eq!(nodes.rank(i), 1 + left_height.max(right_height), "node {i} height");
        for child in [left, right].into_iter().filter(|&c| c != nodes.nil) {
            assert_eq!(nodes.parent(child), i, "node {child} parent");
        }
        1 + left_height.max(right_height)
    }

    fn assert_stays_avl<L: Layout>() {
        let mut arr = pseudo_random(300, 40);
        let n = arr.len();
        let mut logger = NoOpLogger;
        let mut nodes = Nodes::<_, _, L>::new(&mut arr, &mut logger);
        for i in 0..n {
            nodes.attach(i, Avl::LEAF_RANK);
            Avl::inserted(&mut nodes, i);
            assert_avl(&nodes, nodes.root);
        }
        for k in 0..n {
            let (least, parent) = nodes.unlink_min();
            Avl::removed(&mut nodes, parent);
            nodes.place(least, k);
            assert_avl(&nodes, nodes.root);
        }
        assert_eq!(nodes.root, nodes.nil);
    }

    #[test]
    fn stays_an_avl_tree_through_every_insertion_and_extraction() {
        assert_stays_avl::<Packed>();
        assert_stays_avl::<Halves>();
        assert_stays_avl::<Whole>();
    }
}
