//! `FixedAvl`, the classic AVL tree written for slack 1 alone: the balance
//! test is a literal one level and the heights are stored as they are, where
//! `Avl<1>` takes its bound as a parameter and wraps its heights to the rank
//! field. Same decisions, same tree; the measurement below says why both stay.

use array_vis_bench_traits::composable::HasTimeBounds;
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::{Layout, Nodes, LEFT, RIGHT};
use crate::tree_sort::Tree;

// A tree of 2^19 - 1 nodes is at most 27 levels tall at slack 1, so a height
// never outgrows even the 7 bit rank field of the one word layout and needs no
// wrapping. Timed against `Avl<1>` in one binary on one pinned core over
// random input, this tree sorted 6% faster at 2^14 keys and 5% at 2^17, with
// the paired runs within 0.5% and 2% of each other, and the two sorts made the
// same comparisons and writes. The wrapped heights account for about 2.5% of
// the gap and the parameterised code for the rest. At 2^20 the machine's own
// noise, 25% between runs, hid it, the medians again putting this tree 5% ahead.
pub struct FixedAvl;

impl Tree for FixedAvl {
    const LEAF_RANK: usize = 1;

    fn inserted<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, node: usize) {
        retrace(nodes, nodes.parent(node));
    }

    fn removed<T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout>(nodes: &mut Nodes<'_, T, U, L>, parent: usize) {
        retrace(nodes, parent);
    }
}

impl HasTimeBounds for FixedAvl {
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
    use crate::avl::Avl;
    use crate::nodes::{Halves, Packed, Whole, PARENT};
    use crate::test_input::pseudo_random;
    use sort_logger::NoOpLogger;

    // Both trees hold the same links and the same heights at every index.
    fn assert_same_tree<L: Layout>(fixed: &Nodes<usize, NoOpLogger, L>, wrapped: &Nodes<usize, NoOpLogger, L>, n: usize) {
        assert_eq!(fixed.root, wrapped.root);
        for i in 0..n {
            for field in [PARENT, LEFT, RIGHT] { assert_eq!(fixed.link(i, field), wrapped.link(i, field), "node {i} field {field}"); }
            assert_eq!(fixed.rank(i), wrapped.rank(i), "node {i} height");
        }
    }

    // `FixedAvl` and `Avl<1>` fed the same keys stay the same tree through
    // every attach and every leftmost deletion.
    fn assert_matches_avl_of_slack_one<L: Layout>() {
        let n = 300;
        let ascending: Vec<usize> = (0..n).collect();
        for input in [pseudo_random(n, 40), pseudo_random(n, 1 << 20), ascending.clone(), ascending.into_iter().rev().collect()] {
            let (mut fixed_arr, mut wrapped_arr) = (input.clone(), input);
            let (mut fixed_meta, mut wrapped_meta) = (vec![0; L::words(n)], vec![0; L::words(n)]);
            let (mut fixed_logger, mut wrapped_logger) = (NoOpLogger, NoOpLogger);
            let mut fixed = Nodes::<_, _, L>::over(n, &mut fixed_meta, &mut fixed_logger);
            let mut wrapped = Nodes::<_, _, L>::over(n, &mut wrapped_meta, &mut wrapped_logger);
            for i in 0..n {
                fixed.attach(i, FixedAvl::LEAF_RANK, &fixed_arr[..]);
                FixedAvl::inserted(&mut fixed, i);
                wrapped.attach(i, Avl::<1>::LEAF_RANK, &wrapped_arr[..]);
                Avl::<1>::inserted(&mut wrapped, i);
                assert_same_tree(&fixed, &wrapped, n);
            }
            for k in 0..n {
                let ((least, parent), (wrapped_least, wrapped_parent)) = (fixed.unlink_min(), wrapped.unlink_min());
                assert_eq!((least, parent), (wrapped_least, wrapped_parent));
                FixedAvl::removed(&mut fixed, parent);
                Avl::<1>::removed(&mut wrapped, parent);
                fixed.place(&mut fixed_arr, least, k);
                wrapped.place(&mut wrapped_arr, least, k);
                assert_same_tree(&fixed, &wrapped, n);
            }
        }
    }

    #[test]
    fn builds_the_tree_avl_of_slack_one_builds_in_every_layout() {
        assert_matches_avl_of_slack_one::<Packed>();
        assert_matches_avl_of_slack_one::<Halves>();
        assert_matches_avl_of_slack_one::<Whole>();
    }
}
