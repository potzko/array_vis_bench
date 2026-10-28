//! `Naive`, the tree of the tree sort family that never rebalances. The shape
//! is whatever the attach order gives, a path on sorted input, so the sort is
//! quadratic in the worst case and n log n on random input.

use array_vis_bench_traits::composable::HasTimeBounds;
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::nodes::Nodes;
use crate::tree_sort::Tree;

pub struct Naive;

impl Tree for Naive {
    const LEAF_RANK: usize = 0;

    fn inserted<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_nodes: &mut Nodes<'_, T, U>, _node: usize) {}

    fn removed<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_nodes: &mut Nodes<'_, T, U>, _parent: usize) {}
}

// Attaching n keys costs the sum of their depths: a path for sorted or all
// equal input, about n log n for a random order or a balanced one, so the
// best case is no better than n log n either.
impl HasTimeBounds for Naive {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{LEFT, RIGHT};
    use sort_logger::NoOpLogger;

    #[test]
    fn sorted_input_builds_a_right_path_with_no_rotation_and_no_rank() {
        let mut arr: Vec<usize> = (0..64).collect();
        let mut logger = NoOpLogger;
        let mut nodes = Nodes::new(&mut arr, &mut logger);
        for i in 0..64 {
            nodes.attach(i, Naive::LEAF_RANK);
            Naive::inserted(&mut nodes, i);
        }
        assert_eq!(nodes.root, 0);
        for i in 0..64 {
            assert_eq!(nodes.link(i, LEFT), nodes.nil, "node {i} has a left child");
            assert_eq!(nodes.link(i, RIGHT), if i == 63 { nodes.nil } else { i + 1 }, "node {i} right link");
            assert_eq!(nodes.rank(i), 0, "node {i} carries a rank it never reads");
        }
    }
}
