//! `RodGaps<S>`: a branching strategy's gap chain as a Shell sort gap
//! sequence. Shell sort over it is rod sort with the insertion merge, run one
//! gap at a time over every chain instead of one chain at a time.

use std::marker::PhantomData;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::HasTimeBounds;
use shell_sort_lib::GapSequence;

use crate::shell_branching::BranchingStrategy;

pub struct RodGaps<S: BranchingStrategy> {
    _phantom: PhantomData<S>,
}

impl<S: BranchingStrategy> GapSequence for RodGaps<S> {
    const NAME: &'static str = "rod";
    const BIG_O: &'static str = "O(N^2)";

    // The leftmost path of the tree `RodSort` builds over `len`: each level's
    // jump, then that jump times `inter` where the length gate holds, down to
    // the cut. Every node at a depth runs the same passes when the tree is
    // uniform; elsewhere a sibling that cuts a level earlier is the difference.
    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps = Vec::new();
        let (mut jump, mut virtual_len) = (1, len);
        while virtual_len > 1 {
            gaps.push(jump);
            if S::should_cut(virtual_len) { break; }
            let inter = S::intermediate(virtual_len);
            if inter > 0 && virtual_len >= inter * 16 { gaps.push(jump * inter); }
            let branch = S::branch(virtual_len);
            jump *= branch;
            virtual_len = virtual_len.div_ceil(branch);
        }
        gaps.reverse();
        gaps
    }
}

// The insertion merge's bounds: a pass per level, N^2 at most in all.
impl<S: BranchingStrategy> HasTimeBounds for RodGaps<S> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::product(Complexity::N1, S::DEPTH);
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}

#[cfg(test)]
mod tests {
    use shell_sort_lib::ShellSort;

    use super::*;
    use crate::testing::Compares;
    use crate::{Classic, InsertionMerge, Optimised, Parity3, RodSort};

    fn compares(sort: fn(&mut [usize], &mut Compares), n: usize) -> usize {
        let mut arr: Vec<usize> = (0..n).map(|i| i * 7919 % n).collect();
        let mut compares = Compares(0);
        sort(&mut arr, &mut compares);
        compares.0
    }

    #[test]
    fn rod_gaps_compare_as_rod_sort_does_where_its_tree_is_uniform() {
        assert_eq!(compares(ShellSort::<RodGaps<Classic>>::sort, 1 << 10), compares(RodSort::<Classic, InsertionMerge>::sort, 1 << 10));
        assert_eq!(compares(ShellSort::<RodGaps<Parity3>>::sort, 729), compares(RodSort::<Parity3, InsertionMerge>::sort, 729));
        assert_eq!(compares(ShellSort::<RodGaps<Optimised>>::sort, 1 << 15), compares(RodSort::<Optimised, InsertionMerge>::sort, 1 << 15));
    }
}
