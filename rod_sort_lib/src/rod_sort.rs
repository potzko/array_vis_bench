use std::marker::PhantomData;

use super::merge::{insertion_sort_jump, RodMerge};
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use sort_logger::SortLogger;
use crate::shell_branching::BranchingStrategy;

/// Rod sort — recursive divide-and-conquer sort generalised over a branching
/// strategy `S` and a final-merge strategy `M`.
///
/// Each recursive call works on the virtual sub-array defined by the slice
/// `arr[offset..]` with elements spaced `jump` apart.  At each level:
///
///   1. Compute `branch = S::branch(virtual_len)`.
///   2. Recurse into each of the `branch` interleaved sub-sub-arrays at stride `jump * branch`.
///   3. Hand off to `M::merge`, forwarding `branch` and the strategy's
///      `intermediate(virtual_len)` factor (already gated for the
///      `virtual_len >= 16·inter` heuristic). Whether the merge actually
///      uses `inter` for a coarser pre-pass is up to `M` — `InsertionMerge`
///      does; `AuxMerge` ignores it, since a coarse pre-sort would scramble
///      the branch-substreams it expects.
///
/// When `M::NEEDS_AUX` is true, a single aux buffer of size N is allocated
/// at the top level and threaded through the recursion — every shallower
/// merge needs strictly less than the top-level merge, and the recursion
/// is sequential, so no per-level allocation is needed.
pub struct RodSort<S: BranchingStrategy, M: RodMerge> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: BranchingStrategy, M: RodMerge> RodSort<S, M> {
    #[inline]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.is_empty() {
            return;
        }
        if M::NEEDS_AUX {
            let mut aux = logger.create_aux_arr_t(arr.len());
            Self::sort_rec(arr, 1, &mut aux, logger);
            logger.free_aux_arr_t(&aux);
        } else {
            Self::sort_rec(arr, 1, &mut [], logger);
        }
    }

    fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        aux: &mut [T],
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
                Self::sort_rec(&mut arr[offset..], jump * branch, aux, logger);
            }
        }

        let inter = S::intermediate(virtual_len);
        let effective_inter = if inter > 0 && virtual_len >= inter * 16 {
            inter
        } else {
            0
        };

        M::merge(arr, aux, jump, branch, effective_inter, logger);
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// RodSort's complexity is dominated by the merge strategy — the
// branching strategy only affects constants. Stability requires both
// the merge and the branching to be stable.
// Space is `LOG_N` (recursion stack) when the merge is in-place; `N1`
// when AuxMerge allocates the top-level buffer.

impl<S, M> HasTimeBounds for RodSort<S, M>
where
    S: BranchingStrategy,
    M: RodMerge + HasTimeBounds,
{
    const WORST: Complexity = M::WORST;
    const BEST: Complexity = M::BEST;
    const AVERAGE: Complexity = M::AVERAGE;
}

impl<S, M> HasSpace for RodSort<S, M>
where
    S: BranchingStrategy,
    M: RodMerge + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(Complexity::LOG_N, M::SPACE);
}

impl<S, M> HasStability for RodSort<S, M>
where
    S: BranchingStrategy + HasStability,
    M: RodMerge + HasStability,
{
    const STABLE: bool = S::STABLE && M::STABLE;
}

