use std::marker::PhantomData;

use super::merge::{insertion_sort_jump, AuxMerge, InsertionMerge, RodMerge, TournamentMerge};
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
/// The merge's aux buffer and index scratch, as `M::aux_size` and
/// `M::index_size` ask for them, are allocated once at the top level and
/// threaded through the recursion: every shallower merge needs no more than
/// the top-level merge, and the recursion is sequential, so no per-level
/// allocation is needed.
pub struct RodSort<S: BranchingStrategy, M: RodMerge> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: BranchingStrategy, M: RodMerge> RodSort<S, M> {
    #[inline]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let (aux_len, index_len) = (M::aux_size(n), M::index_size(S::branch(n)));
        let mut aux = if aux_len > 0 { logger.create_aux_arr_t(aux_len) } else { Vec::new() };
        let mut index = if index_len > 0 { logger.create_aux_arr(index_len) } else { Vec::new() };
        Self::sort_rec(arr, 1, &mut aux, &mut index, logger);
        if index_len > 0 { logger.free_aux_arr(&index); }
        if aux_len > 0 { logger.free_aux_arr_t(&aux); }
    }

    fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        aux: &mut [T],
        index: &mut [usize],
        logger: &mut U,
    ) {
        // The chain is arr[0], arr[jump], ..., so a partial last stride still
        // holds one element. A chain of one is sorted whatever the strategy.
        let virtual_len = arr.len().div_ceil(jump);
        if virtual_len <= 1 {
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
                Self::sort_rec(&mut arr[offset..], jump * branch, aux, index, logger);
            }
        }

        let inter = S::intermediate(virtual_len);
        let effective_inter = if inter > 0 && virtual_len >= inter * 16 {
            inter
        } else {
            0
        };

        M::merge(arr, aux, index, jump, branch, effective_inter, logger);
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

// A loser tree pays at most ceil(log2 b) compares per element at a level of
// branch b, and the logs of the branches on a path sum to log2 N whatever the
// strategy, since their product is N.
impl<S: BranchingStrategy> HasTimeBounds for RodSort<S, TournamentMerge> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
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
    use std::cell::RefCell;
    use std::cmp::Ordering;
    use std::marker::PhantomData;

    use super::*;
    use crate::{Classic, Optimised};
    use sort_logger::{NoOpLogger, SortLog, VisualizerLogger};

    fn buffers_created(sort: fn(&mut [usize], &mut VisualizerLogger<usize>)) -> usize {
        let mut arr: Vec<usize> = (0..4096).rev().collect();
        let mut logger = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        sort(&mut arr, &mut logger);
        logger.log.iter().filter(|e| matches!(e, SortLog::CreateAuxArrT { .. } | SortLog::CreateAuxArr { .. })).count()
    }

    #[test]
    fn a_sort_creates_each_scratch_once_not_per_merge() {
        // 4096 under a branch of 32 is 33 merges, against one aux and one index buffer.
        assert_eq!(buffers_created(RodSort::<Optimised, AuxMerge>::sort), 2);
        assert_eq!(buffers_created(RodSort::<Optimised, InsertionMerge>::sort), 0);
    }

    // Wide on short chains and narrow on long ones, so the index scratch sized
    // from the top level is too small further down.
    struct Narrowing;
    impl BranchingStrategy for Narrowing {
        const NAME: &'static str = "narrowing";
        const DEPTH: Complexity = Complexity::LOG_N;
        const PATH_BRANCHING: Complexity = Complexity::LOG_N;
        fn should_cut(virtual_len: usize) -> bool { virtual_len < 4 }
        fn branch(virtual_len: usize) -> usize { if virtual_len > 64 { 2 } else { 8 } }
    }

    #[test]
    fn a_merge_handed_too_little_index_scratch_brings_its_own() {
        let sorts: [fn(&mut [usize], &mut NoOpLogger); 2] =
            [RodSort::<Narrowing, AuxMerge>::sort, RodSort::<Narrowing, TournamentMerge>::sort];
        for sort in sorts {
            let mut arr: Vec<usize> = (0..500).map(|i| i * 7919 % 500).collect();
            sort(&mut arr, &mut NoOpLogger);
            assert!(arr.windows(2).all(|pair| pair[0] <= pair[1]));
        }
    }

    // Orders by `key` alone, so `tag` shows where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Keyed { key: usize, tag: usize }
    impl PartialEq for Keyed { fn eq(&self, other: &Self) -> bool { self.key == other.key } }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
    impl Ord for Keyed { fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) } }

    thread_local! { static CUTS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) }; }

    // Splits in two and records every length it cuts. Cutting at four or less
    // means no split below the root makes a chain of one, which the driver
    // would skip, so the cut lengths account for every element.
    struct Recorded;
    impl BranchingStrategy for Recorded {
        const NAME: &'static str = "recorded";
        const DEPTH: Complexity = Complexity::LOG_N;
        const PATH_BRANCHING: Complexity = Complexity::LOG_N;
        fn should_cut(virtual_len: usize) -> bool {
            let cut = virtual_len <= 4;
            if cut { CUTS.with(|cuts| cuts.borrow_mut().push(virtual_len)); }
            cut
        }
        fn branch(_: usize) -> usize { 2 }
    }

    #[test]
    fn every_leaf_is_told_how_many_elements_its_chain_holds() {
        for n in 2..64 {
            CUTS.with(|cuts| cuts.borrow_mut().clear());
            let mut arr: Vec<usize> = (0..n).rev().collect();
            RodSort::<Recorded, InsertionMerge>::sort(&mut arr, &mut NoOpLogger);
            assert_eq!(CUTS.with(|cuts| cuts.borrow().iter().sum::<usize>()), n, "leaf lengths for n = {n}");
        }
    }

    #[test]
    fn a_rod_sort_reorders_equal_keys_and_does_not_claim_stability() {
        let mut arr = [(9, 0), (1, 1), (1, 2), (9, 3)].map(|(key, tag)| Keyed { key, tag });
        RodSort::<Classic, InsertionMerge>::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr.map(|k| k.tag), [2, 1, 0, 3]);
        assert!(!<RodSort<Classic, InsertionMerge> as HasStability>::STABLE);
    }
}

