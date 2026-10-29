use std::marker::PhantomData;

use super::merge::{insertion_sort_jump, AuxMerge, InsertionMerge, RodMerge};
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

array_vis_bench_traits::direct_sort!(RodSort<S, M>; S: BranchingStrategy, M: RodMerge);

// ── Composable annotations ──────────────────────────────────────────
//
// Space is `LOG_N` (recursion stack) when the merge is in-place; `N1`
// when AuxMerge allocates the top-level buffer.

// On sorted input every pass compares each element once with its chain
// predecessor, so the best case is N per level. A level with jump `j` holds `j`
// chains of N / j, so its insertion passes cost at most N^2 / j, and the levels
// sum to N^2. The average is left at that bound: the gaps form a divisor chain,
// Shell's own powers of two among them, and nothing tighter is shown here.
impl<S: BranchingStrategy> HasTimeBounds for RodSort<S, InsertionMerge> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::product(Complexity::N1, S::DEPTH);
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}

// Each output scans every live chain head, so a level costs N times its branch
// whatever the input, and the levels sum to N times the path branching. The
// two copies per level are N times the depth, which that already covers.
impl<S: BranchingStrategy> HasTimeBounds for RodSort<S, AuxMerge> {
    const WORST: Complexity = Complexity::product(Complexity::N1, S::PATH_BRANCHING);
    const BEST: Complexity = Self::WORST;
    const AVERAGE: Complexity = Self::WORST;
}

impl<S, M> HasSpace for RodSort<S, M>
where
    S: BranchingStrategy,
    M: RodMerge + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(Complexity::LOG_N, M::SPACE);
}

// Splitting by position loses the input order of equal keys whatever the
// merge: in `[x, a1, a2, y]` the even chain sorts to `[a2, x]` and the odd one
// to `[a1, y]`, and nothing a merge can see says `a1` came first.
impl<S: BranchingStrategy, M: RodMerge> HasStability for RodSort<S, M> {
    const STABLE: bool = false;
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;
    use crate::Classic;
    use sort_logger::NoOpLogger;

    // Orders by `key` alone, so `tag` shows where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Keyed { key: usize, tag: usize }
    impl PartialEq for Keyed { fn eq(&self, other: &Self) -> bool { self.key == other.key } }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
    impl Ord for Keyed { fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) } }

    #[test]
    fn a_rod_sort_reorders_equal_keys_and_does_not_claim_stability() {
        let mut arr = [(9, 0), (1, 1), (1, 2), (9, 3)].map(|(key, tag)| Keyed { key, tag });
        RodSort::<Classic, InsertionMerge>::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr.map(|k| k.tag), [2, 1, 0, 3]);
        assert!(!<RodSort<Classic, InsertionMerge> as HasStability>::STABLE);
    }
}

