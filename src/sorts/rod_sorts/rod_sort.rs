use std::marker::PhantomData;

use super::merge::{insertion_sort_jump, RodMerge};
use crate::traits::log_traits::SortLogger;
use crate::utils::shell_branching::BranchingStrategy;

/// Rod sort — recursive divide-and-conquer sort generalised over a branching
/// strategy `S` and a final-merge strategy `M`.
///
/// Each recursive call works on the virtual sub-array defined by the slice
/// `arr[offset..]` with elements spaced `jump` apart.  At each level:
///
///   1. Compute `branch = S::branch(virtual_len)`.
///   2. Recurse into each of the `branch` interleaved sub-sub-arrays at stride `jump * branch`.
///   3. Optionally do an intermediate insertion-sort pass at `S::intermediate(virtual_len)`.
///   4. Merge the `branch` sorted sub-streams via `M::merge` at stride `jump`.
pub struct RodSort<S: BranchingStrategy, M: RodMerge> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: BranchingStrategy, M: RodMerge> RodSort<S, M> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if !arr.is_empty() {
            Self::sort_rec(arr, 1, logger);
        }
    }

    fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        logger: &mut U,
    ) {
        let virtual_len = arr.len() / jump;
        if virtual_len == 0 {
            return;
        }
        if S::should_cut(virtual_len) {
            // Base case: unsorted strided data with no pre-existing merge
            // structure — always insertion-sort, regardless of `M`.
            insertion_sort_jump(arr, jump, logger);
            return;
        }

        let branch = S::branch(virtual_len);
        for i in 0..branch {
            let offset = jump * i;
            if offset < arr.len() {
                Self::sort_rec(&mut arr[offset..], jump * branch, logger);
            }
        }

        let inter = S::intermediate(virtual_len);
        if inter > 0 && virtual_len >= inter * 16 {
            for i in 0..inter {
                let offset = jump * i;
                if offset < arr.len() {
                    insertion_sort_jump(&mut arr[offset..], jump * inter, logger);
                }
            }
        }

        M::merge(arr, jump, branch, logger);
    }
}
