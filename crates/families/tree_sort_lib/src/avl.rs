//! `Avl<SLACK>`, the height balanced tree of the tree sort family, relaxed by
//! a slack: a node is acceptable while its children's heights differ by at most
//! `SLACK`, so `Avl<1>` is the classic AVL tree. After each attach and each
//! leftmost deletion it retraces towards the root, refreshing heights and
//! rotating wherever two siblings differ by more than the slack. The rank word
//! of the node store holds the height, modulo the width of the rank field.

use std::marker::PhantomData;

use array_vis_bench_traits::composable::HasTimeBounds;
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::{Layout, Nodes, LEFT, RIGHT};
use crate::tree_sort::Tree;

pub struct Avl<const SLACK: usize>;

impl<const SLACK: usize> Avl<SLACK> {
    // The first imbalance a node may not keep.
    const THRESHOLD: usize = SLACK + 1;
}

impl<const SLACK: usize> Tree for Avl<SLACK> {
    const LEAF_RANK: usize = 1;

    fn inserted<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, node: usize) {
        retrace::<SLACK, T, U, L>(nodes, nodes.parent(node));
    }

    fn removed<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, parent: usize) {
        retrace::<SLACK, T, U, L>(nodes, parent);
    }
}

// A tree with n nodes and height h has at least N(h) nodes, where
// N(h) = N(h - 1) + N(h - 1 - SLACK) + 1: the root, a child of height h - 1,
// and the other child at the lowest height the slack allows. N grows like r^h
// for the root r > 1 of r^(SLACK + 1) = r^SLACK + 1, so for a fixed slack the
// height is O(log n) and every attach and deletion retraces O(log n) nodes.
// The slack sets the constant: the tallest tree of 2^19 - 1 nodes is 27 levels
// at slack 1, 33 at 2, 45 at 4, 64 at 8, 97 at 16 and 151 at 32.
impl<const SLACK: usize> HasTimeBounds for Avl<SLACK> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

// Heights as the rank field of layout `L` holds them, modulo 2^RANK_BITS. The
// one word layout has 7 bits, so 127 levels, and a slack of 32 can grow a tree
// of 2^19 - 1 nodes to 151. The tree only ever compares heights at most
// SLACK + 1 levels apart, a node's two children or a child's two, so the
// difference taken modulo 2^RANK_BITS and read as signed is the real one while
// SLACK + 1 is under half of 2^RANK_BITS, checked when a tree is built. The
// missing child has height 0 and a node never sits next to it unless its own
// height is at most SLACK + 1, so a stored 0 is never mistaken for a wrapped
// height.
struct Height<L>(PhantomData<L>);

impl<L: Layout> Height<L> {
    const SHIFT: u32 = usize::BITS - L::RANK_BITS as u32;
    const MASK: usize = usize::MAX >> Self::SHIFT;

    // How many levels `a` stands above `b`, negative when it stands below.
    fn lead(a: usize, b: usize) -> isize { (a.wrapping_sub(b) << Self::SHIFT) as isize >> Self::SHIFT }

    // The height of a node over children of heights `a` and `b`.
    fn above(a: usize, b: usize) -> usize { (1 + if Self::lead(a, b) >= 0 { a } else { b }) & Self::MASK }

    // Whether two heights `gap` levels apart are told apart from their stored values.
    const fn tells_apart(gap: usize) -> bool { gap < 1 << (L::RANK_BITS - 1) }
}

// Walk up from `x` restoring heights and balance. Stops at the first node
// whose subtree kept its height, because the heights above it are then
// untouched: an insertion stops at its first rotation, a deletion may rotate
// at several levels.
//
// A node never exceeds the threshold by more than reaching it. One attach or
// one deletion moves the height of a subtree by at most one, and a rotation
// below leaves its subtree at the height it had before the operation or one
// less on a deletion, so every sibling pair moves by at most one from a pair
// that was within the slack.
fn retrace<const SLACK: usize, T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(
    nodes: &mut Nodes<'_, T, U, L>,
    mut x: usize,
) {
    const { assert!(Height::<L>::tells_apart(SLACK + 1), "the rank field is too narrow for this slack") };
    while x != nodes.nil {
        let parent = nodes.parent(x);
        let before = nodes.rank(x);
        let (left, right) = (nodes.rank(nodes.link(x, LEFT)), nodes.rank(nodes.link(x, RIGHT)));
        let after = if Height::<L>::lead(left, right).unsigned_abs() >= Avl::<SLACK>::THRESHOLD {
            let top = rebalance(nodes, x);
            nodes.rank(top)
        } else {
            let after = Height::<L>::above(left, right);
            if after != before { nodes.set_rank(x, after); }
            after
        };
        if after == before { return; }
        x = parent;
    }
}

// Restore balance at `x`, whose children differ by exactly SLACK + 1, with one
// rotation towards its lighter side, after first rotating the heavy child when
// its inner grandchild is the taller, the double rotation case. Let the light
// child have height L, so the heavy child has L + SLACK + 1 and its children,
// the inner `a` and the outer `b`, are within SLACK of each other with the
// taller at L + SLACK.
//
// A single rotation leaves `x` over L and `a`, within SLACK since a >= L, and
// the heavy child over `x` and `b`, within SLACK when a <= b, as then the
// heights 1 + max(L, a) and b differ by 1 + a - b, from 1 - SLACK to 1. It
// fails only when a > b, so a = L + SLACK, and then the double rotation puts
// the inner child on top. Its own children c and d have max(c, d) = L + SLACK
// - 1 and are within SLACK of each other, which leaves x over L and c, the
// old heavy child over d and b, and the top over those two, each pair within
// SLACK. An insertion never reaches a = b, the grown side being strictly taller.
fn rebalance<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, x: usize) -> usize {
    let left_heavy = Height::<L>::lead(nodes.rank(nodes.link(x, LEFT)), nodes.rank(nodes.link(x, RIGHT))) > 0;
    let (side, other) = if left_heavy { (RIGHT, LEFT) } else { (LEFT, RIGHT) };
    let heavy = nodes.link(x, other);
    if Height::<L>::lead(nodes.rank(nodes.link(heavy, side)), nodes.rank(nodes.link(heavy, other))) > 0 {
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
    let height = Height::<L>::above(nodes.rank(nodes.link(i, LEFT)), nodes.rank(nodes.link(i, RIGHT)));
    nodes.set_rank(i, height);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{Halves, Packed, Whole, PARENT, RANK};
    use crate::test_input::{each_slack, pseudo_random};
    use sort_logger::NoOpLogger;

    // The stored height of every node is right modulo the rank field, its
    // children differ by at most SLACK levels and each child points back at
    // it, so the tree really is a relaxed AVL tree and the bound holds.
    // Returns the true height.
    fn assert_within_slack<const SLACK: usize, T: Ord + Copy, L: Layout>(nodes: &Nodes<T, NoOpLogger, L>, i: usize) -> usize {
        if i == nodes.nil { return 0; }
        let (left, right) = (nodes.link(i, LEFT), nodes.link(i, RIGHT));
        let heights = [left, right].map(|child| assert_within_slack::<SLACK, T, L>(nodes, child));
        assert!(heights[0].abs_diff(heights[1]) <= SLACK, "node {i} is out of balance by more than {SLACK}");
        let height = 1 + heights[0].max(heights[1]);
        assert_eq!(nodes.rank(i), height & Height::<L>::MASK, "node {i} height {height}");
        for child in [left, right].into_iter().filter(|&c| c != nodes.nil) {
            assert_eq!(nodes.parent(child), i, "node {child} parent");
        }
        height
    }

    // Every attach and every leftmost deletion, with the whole tree checked
    // after each.
    fn assert_stays_within_slack<const SLACK: usize, L: Layout>() {
        let n = 300;
        let ascending: Vec<usize> = (0..n).collect();
        let inputs = [pseudo_random(n, 40), pseudo_random(n, 1 << 20), ascending.clone(), ascending.into_iter().rev().collect()];
        for mut arr in inputs {
            let (mut meta, mut logger) = (vec![0; L::words(n)], NoOpLogger);
            let mut nodes = Nodes::<_, _, L>::over(&mut arr, &mut meta, &mut logger);
            for i in 0..n {
                nodes.attach(i, Avl::<SLACK>::LEAF_RANK);
                Avl::<SLACK>::inserted(&mut nodes, i);
                assert_within_slack::<SLACK, _, L>(&nodes, nodes.root);
            }
            for k in 0..n {
                let (least, parent) = nodes.unlink_min();
                Avl::<SLACK>::removed(&mut nodes, parent);
                nodes.place(least, k);
                assert_within_slack::<SLACK, _, L>(&nodes, nodes.root);
            }
            assert_eq!(nodes.root, nodes.nil);
        }
    }

    #[test]
    fn stays_within_its_slack_through_every_insertion_and_extraction() {
        each_slack!(assert_stays_within_slack, Packed);
        each_slack!(assert_stays_within_slack, Halves);
        each_slack!(assert_stays_within_slack, Whole);
    }

    // Nodes in the sparsest tree of a height, the N(h) of the time bound.
    fn sparsest_size(slack: usize, height: usize) -> usize {
        let grow = |mut sizes: Vec<usize>, h: usize| {
            sizes.push(1 + sizes[h - 1] + sizes[(h - 1).saturating_sub(slack)]);
            sizes
        };
        (1..=height).fold(vec![0], grow)[height]
    }

    // The tallest tree `slack` allows over `n` nodes.
    fn tallest(slack: usize, n: usize) -> usize { (0..).take_while(|&h| sparsest_size(slack, h) <= n).last().unwrap() }

    #[test]
    fn the_tallest_trees_a_word_per_node_can_hold_by_slack() {
        let heights = [1, 2, 4, 8, 16, 32].map(|slack| tallest(slack, Packed::REACH));
        assert_eq!(heights, [27, 33, 45, 64, 97, 151]);
        assert!(heights[..5].iter().all(|&h| h < 1 << Packed::RANK_BITS));
        assert!(heights[5] >= 1 << Packed::RANK_BITS, "a slack of 32 is what outgrows the 7 bit rank field");
    }

    // A test layout whose rank field holds BITS bits and refuses more, in
    // release builds too, so a height that is not wrapped is caught.
    struct Narrow<const BITS: usize>;

    impl<const BITS: usize> Layout for Narrow<BITS> {
        const REACH: usize = Halves::REACH;
        const RANK_BITS: usize = BITS;

        fn words(n: usize) -> usize { Halves::words(n) }

        fn get(meta: &[usize], i: usize, field: usize) -> usize { Halves::get(meta, i, field) }

        fn set<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, i: usize, field: usize, value: usize) {
            assert!(field != RANK || value >> BITS == 0, "rank {value} does not fit {BITS} bits");
            Halves::set::<T, U>(meta, logger, i, field, value);
        }
    }

    // Hang the sparsest tree of `height` under `parent` with no rebalancing:
    // the taller child one level down, the shorter `slack` levels lower than
    // that or empty, the taller side alternating with depth so that deletions
    // from the left meet both a light and a heavy side. Nodes are numbered in
    // pre order and keyed in order, from 1. Returns the subtree root.
    fn hang_sparsest<const SLACK: usize, L: Layout>(
        nodes: &mut Nodes<usize, NoOpLogger, L>,
        (height, depth, parent): (usize, usize, usize),
        (next_node, next_key): (&mut usize, &mut usize),
    ) -> usize {
        if height == 0 { return nodes.nil; }
        let i = *next_node;
        *next_node += 1;
        let (tall, short) = (height - 1, (height - 1).saturating_sub(SLACK));
        let (left_height, right_height) = if depth % 2 == 0 { (tall, short) } else { (short, tall) };
        let left = hang_sparsest::<SLACK, L>(nodes, (left_height, depth + 1, i), (next_node, next_key));
        nodes.set_key(i, *next_key);
        *next_key += 1;
        let right = hang_sparsest::<SLACK, L>(nodes, (right_height, depth + 1, i), (next_node, next_key));
        for (field, to) in [(PARENT, parent), (LEFT, left), (RIGHT, right)] { nodes.set_link(i, field, to); }
        nodes.set_rank(i, height & Height::<L>::MASK);
        i
    }

    // The sparsest tree of `height`, then 40 more keys attached through the
    // retrace and every node taken out through it. The whole tree is checked
    // after each of the first 40 steps of both and about 50 times beyond, the
    // sparsest tree being too big to check at every step.
    fn assert_tall_tree_stays_exact<const SLACK: usize, L: Layout>(height: usize) {
        let (n, extra) = (sparsest_size(SLACK, height), 40);
        let stride = (n + extra) / 50 + 1;
        let (mut arr, mut meta, mut logger) = (vec![0; n + extra], vec![0; L::words(n + extra)], NoOpLogger);
        let mut nodes = Nodes::<_, _, L>::over(&mut arr, &mut meta, &mut logger);
        let nil = nodes.nil;
        nodes.root = hang_sparsest::<SLACK, L>(&mut nodes, (height, 0, nil), (&mut 0, &mut 1));
        assert_eq!(assert_within_slack::<SLACK, _, L>(&nodes, nodes.root), height, "the sparsest tree is as tall as asked");
        for (i, key) in (n..n + extra).zip(pseudo_random(extra, n)) {
            nodes.set_key(i, key);
            nodes.attach(i, Avl::<SLACK>::LEAF_RANK);
            Avl::<SLACK>::inserted(&mut nodes, i);
            assert_within_slack::<SLACK, _, L>(&nodes, nodes.root);
        }
        for k in 0..n + extra {
            let (least, parent) = nodes.unlink_min();
            Avl::<SLACK>::removed(&mut nodes, parent);
            nodes.place(least, k);
            if k < 40 || k % stride == 0 { assert_within_slack::<SLACK, _, L>(&nodes, nodes.root); }
        }
        assert_eq!(nodes.root, nodes.nil);
    }

    // Trees taller than the rank field has values keep exact heights: first
    // through narrow fields where a small tree wraps them, then at every slack
    // through the word per node layout, tallest as 2^19 - 1 nodes allow.
    #[test]
    fn a_tree_taller_than_its_rank_field_keeps_exact_heights() {
        assert_tall_tree_stays_exact::<1, Narrow<3>>(12);
        assert_tall_tree_stays_exact::<2, Narrow<3>>(12);
        assert_tall_tree_stays_exact::<1, Narrow<4>>(20);
        assert_tall_tree_stays_exact::<2, Narrow<4>>(20);
        assert_tall_tree_stays_exact::<4, Narrow<4>>(20);
        assert_tall_tree_stays_exact::<4, Narrow<5>>(36);
        assert_tall_tree_stays_exact::<8, Narrow<5>>(36);
        assert_tall_tree_stays_exact::<1, Packed>(tallest(1, Packed::REACH - 40));
        assert_tall_tree_stays_exact::<2, Packed>(tallest(2, Packed::REACH - 40));
        assert_tall_tree_stays_exact::<4, Packed>(tallest(4, Packed::REACH - 40));
        assert_tall_tree_stays_exact::<8, Packed>(tallest(8, Packed::REACH - 40));
        assert_tall_tree_stays_exact::<16, Packed>(tallest(16, Packed::REACH - 40));
        assert_tall_tree_stays_exact::<32, Packed>(tallest(32, Packed::REACH - 40));
    }
}
