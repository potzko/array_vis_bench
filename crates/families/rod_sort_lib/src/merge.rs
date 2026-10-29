//! Final-merge strategy for [`super::rod_sort::RodSort`].
//!
//! After recursing on each of `branch` interleaved sub-streams (at stride
//! `jump * branch`), the slice holds `branch` already-sorted sub-streams
//! interleaved at stride `jump`. The merge step turns them into a single
//! sorted stride-`jump` sequence.
//!
//! - [`InsertionMerge`]: the original strategy — strided insertion sort
//!   over the full slice at stride `jump`. Cheap when sub-streams are
//!   short and nearly-sorted, but O(N · stream_length) in the worst case.
//! - [`AuxMerge`]: a `branch`-way merge through an aux buffer that scans
//!   every chain head for each output, so `branch - 1` compares per element
//!   per pass, plus N reads and writes for the buffer round trip. Merge sort
//!   overall only while the branch stays constant.
//!
//! ## Aux buffer ownership
//!
//! The top-level merge needs an aux buffer of size N (jump=1, virtual_len=N).
//! Every shallower merge needs strictly less. The recursion is sequential,
//! so a single N-sized buffer suffices for the entire sort and is allocated
//! once at the top, see [`super::rod_sort::RodSort::sort`]. The chain heads
//! are one index buffer the same way, sized by the top level's branch.
//! [`InsertionMerge`] asks for neither, so it allocates nothing.

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::HasSpace;
use sort_logger::SortLogger;

pub trait RodMerge {
    const NAME: &'static str;

    /// Aux elements the merges of a sort over `n` need, allocated once by the
    /// driver and shared by every merge. 0 for a merge that works in place.
    #[inline]
    fn aux_size(_n: usize) -> usize {
        0
    }

    /// Index scratch a merge of `branch` chains needs. The driver sizes it by
    /// the top level's branch, the widest a sort meets while the strategy's
    /// branch grows with the length; a merge handed less brings its own.
    #[inline]
    fn index_size(_branch: usize) -> usize {
        0
    }

    /// Merge the `branch` already-sorted sub-streams at stride `jump` into
    /// a single sorted stride-`jump` sequence over `arr`.
    ///
    /// `aux` and `index` are the buffers [`Self::aux_size`] and
    /// [`Self::index_size`] asked for, or empty when they asked for none. `inter` is the branching strategy's intermediate factor
    /// (already gated by the caller — 0 means skip). The InsertionMerge
    /// uses `inter` for an extra pre-pass at stride `jump * inter`; the
    /// AuxMerge ignores it (it would scramble the branch sub-streams).
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        aux: &mut [T],
        index: &mut [usize],
        jump: usize,
        branch: usize,
        inter: usize,
        logger: &mut U,
    );
}

pub struct InsertionMerge;
impl RodMerge for InsertionMerge {
    const NAME: &'static str = "insertion";

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _aux: &mut [T],
        _index: &mut [usize],
        jump: usize,
        _branch: usize,
        inter: usize,
        logger: &mut U,
    ) {
        // Pre-pass: `inter` interleaved insertion sorts at the coarser
        // stride `jump * inter`. Cheap partial sort that the final
        // stride-`jump` insertion pass picks up where it left off.
        if inter > 0 {
            for i in 0..inter {
                let offset = jump * i;
                if offset < arr.len() {
                    insertion_sort_jump(&mut arr[offset..], jump * inter, logger);
                }
            }
        }
        insertion_sort_jump(arr, jump, logger);
    }
}

pub struct AuxMerge;
impl RodMerge for AuxMerge {
    const NAME: &'static str = "aux";

    #[inline]
    fn aux_size(n: usize) -> usize {
        n
    }

    #[inline]
    fn index_size(branch: usize) -> usize {
        branch
    }

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        aux: &mut [T],
        index: &mut [usize],
        jump: usize,
        branch: usize,
        _inter: usize,
        logger: &mut U,
    ) {
        // Number of elements at stride `jump`: positions 0, jump, 2·jump,
        // ... up to the last one < arr.len(). Matches the iteration range
        // of `insertion_sort_jump`.
        let virtual_len = arr.len().div_ceil(jump);
        if branch <= 1 || virtual_len <= 1 {
            return;
        }
        let aux = &mut aux[..virtual_len];
        let chains = branch.min(virtual_len);
        with_index::<T, U>(index, Self::index_size(chains), logger, |heads, logger| {
            scan_merge(arr, aux, heads, jump, branch, logger)
        });
    }
}

pub struct TournamentMerge;
impl RodMerge for TournamentMerge {
    const NAME: &'static str = "tournament";

    #[inline]
    fn aux_size(n: usize) -> usize {
        n
    }

    #[inline]
    fn index_size(branch: usize) -> usize {
        2 * branch
    }

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        aux: &mut [T],
        index: &mut [usize],
        jump: usize,
        branch: usize,
        _inter: usize,
        logger: &mut U,
    ) {
        let virtual_len = arr.len().div_ceil(jump);
        if branch <= 1 || virtual_len <= 1 {
            return;
        }
        let aux = &mut aux[..virtual_len];
        let chains = branch.min(virtual_len);
        with_index::<T, U>(index, Self::index_size(chains), logger, |index, logger| {
            tournament_merge(arr, aux, index, jump, branch, logger)
        });
    }
}

// Runs `body` on the first `len` cells of `index`, or on a buffer of its own
// when the driver's is shorter, which only a strategy whose branch narrows
// with the length can cause.
fn with_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    index: &mut [usize],
    len: usize,
    logger: &mut U,
    body: impl FnOnce(&mut [usize], &mut U),
) {
    if index.len() >= len {
        body(&mut index[..len], logger);
    } else {
        let mut own = logger.create_aux_arr(len);
        body(&mut own, logger);
        logger.free_aux_arr(&own);
    }
}

// Merges `heads.len()` chains into `aux` by scanning every live head for each
// output, then copies `aux` back at stride `jump`. Chain `s` holds virtual
// positions s, s + branch, s + 2 branch, ..., so its head steps by `branch`.
fn scan_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    aux: &mut [T],
    heads: &mut [usize],
    jump: usize,
    branch: usize,
    logger: &mut U,
) {
    let virtual_len = aux.len();
    for s in 0..heads.len() {
        logger.write_data_u(heads, s, s);
    }
    for write_idx in 0..virtual_len {
        let mut min_s = usize::MAX;
        for s in 0..heads.len() {
            if heads[s] >= virtual_len {
                continue;
            }
            if min_s == usize::MAX || logger.cmp_lt_accross(arr, heads[s] * jump, arr, heads[min_s] * jump) {
                min_s = s;
            }
        }
        logger.write_accross(arr, heads[min_s] * jump, aux, write_idx);
        logger.write_data_u(heads, min_s, heads[min_s] + branch);
    }
    for write_idx in 0..virtual_len {
        logger.write_accross(aux, write_idx, arr, write_idx * jump);
    }
}

// A loser tree over the chain heads. Chain `s` is the leaf `chains + s`,
// `tree[node]` holds the loser of the match at `node` and `tree[0]` the
// overall winner, the chain whose head comes next. A spent chain loses every
// match, so it never wins while another chain has elements left.
struct Tournament<'a, T> {
    arr: &'a [T],
    heads: &'a mut [usize],
    tree: &'a mut [usize],
    jump: usize,
    len: usize,
}

impl<T: Ord + Copy> Tournament<'_, T> {
    fn beats<U: ?Sized + SortLogger<T>>(&self, a: usize, b: usize, logger: &mut U) -> bool {
        match (self.heads[a] < self.len, self.heads[b] < self.len) {
            (false, _) => false,
            (true, false) => true,
            (true, true) => logger.cmp_lt_accross(self.arr, self.heads[a] * self.jump, self.arr, self.heads[b] * self.jump),
        }
    }

    // Plays every match under `node`, leaves each loser at its node and
    // returns the winner.
    fn play<U: ?Sized + SortLogger<T>>(&mut self, node: usize, logger: &mut U) -> usize {
        let chains = self.heads.len();
        if node >= chains { return node - chains; }
        let (left, right) = (self.play(2 * node, logger), self.play(2 * node + 1, logger));
        let (winner, loser) = if self.beats(right, left, logger) { (right, left) } else { (left, right) };
        logger.write_data_u(self.tree, node, loser);
        winner
    }

    // Replays the matches on chain `s`'s path to the root once its head moved.
    fn replay<U: ?Sized + SortLogger<T>>(&mut self, s: usize, logger: &mut U) {
        let mut winner = s;
        let mut node = (self.heads.len() + s) / 2;
        while node >= 1 {
            let held = self.tree[node];
            if self.beats(held, winner, logger) {
                logger.write_data_u(self.tree, node, winner);
                winner = held;
            }
            node /= 2;
        }
        logger.write_data_u(self.tree, 0, winner);
    }
}

// Merges the chains into `aux` through a loser tree, at most ceil(log2 k)
// compares per output for k chains, then copies `aux` back at stride `jump`.
// `index` holds the heads, then the tree.
fn tournament_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    aux: &mut [T],
    index: &mut [usize],
    jump: usize,
    branch: usize,
    logger: &mut U,
) {
    let len = aux.len();
    let (heads, tree) = index.split_at_mut(index.len() / 2);
    for s in 0..heads.len() {
        logger.write_data_u(heads, s, s);
    }
    let mut game = Tournament { arr, heads, tree, jump, len };
    let first = game.play(1, logger);
    logger.write_data_u(game.tree, 0, first);
    for write_idx in 0..len {
        let s = game.tree[0];
        logger.write_accross(game.arr, game.heads[s] * jump, aux, write_idx);
        logger.write_data_u(game.heads, s, game.heads[s] + branch);
        game.replay(s, logger);
    }
    for write_idx in 0..len {
        logger.write_accross(aux, write_idx, arr, write_idx * jump);
    }
}

/// Strided insertion sort: sort `arr` viewed as a stride-`jump` virtual
/// array. Reused for the base case and the intermediate pass — those see
/// unsorted strided data, not merge structure, so they always run this.
///
/// Holds the element and shifts the larger chain predecessors up one slot
/// each, so moving d places costs d + 1 writes where swapping cost 2d.
pub(crate) fn insertion_sort_jump<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (jump..arr.len()).step_by(jump) {
        let held = arr[i];
        let mut slot = i;
        while slot >= jump && !logger.cmp_le_data(arr, slot - jump, held) {
            logger.write(arr, slot, slot - jump);
            slot -= jump;
        }
        if slot != i { logger.write_data(arr, slot, held); }
    }
}

// Time bounds depend on the strategy as much as on the merge, so they sit on
// `RodSort<S, M>` for each merge; a merge carries its own space.

impl HasSpace for InsertionMerge {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasSpace for AuxMerge {
    // Top-level merge allocates an N-sized aux buffer.
    const SPACE: Complexity = Complexity::N1;
}
impl HasSpace for TournamentMerge {
    const SPACE: Complexity = Complexity::N1;
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use sort_logger::{SortLog, VisualizerLogger};

    #[test]
    fn strided_insertion_writes_once_per_place_an_element_moves() {
        // A reversed chain of five at stride 3: the element at chain index i
        // moves i places, i shifts and one write of the held element.
        let mut arr = [5, 0, 0, 4, 0, 0, 3, 0, 0, 2, 0, 0, 1];
        let mut logger = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        insertion_sort_jump(&mut arr, 3, &mut logger);
        assert_eq!(arr, [1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0, 5]);
        let swaps = logger.log.iter().filter(|e| matches!(e, SortLog::Swap { .. })).count();
        let writes = logger.log.iter().filter(|e| matches!(e, SortLog::WriteInArr { .. } | SortLog::WriteData { .. })).count();
        assert_eq!((swaps, writes), (0, 10 + 4));
    }

    // Counts comparisons, so a test can hold a merge to its bound.
    struct Compares(usize);
    impl<T: Copy + Ord> SortLogger<T> for Compares {
        fn log(&mut self, event: SortLog<T>) {
            if matches!(event, SortLog::CmpInArr { .. } | SortLog::CmpData { .. } | SortLog::CmpAcrossArrs { .. }) {
                self.0 += 1;
            }
        }
    }

    // `n` values in `chains` interleaved chains, each chain sorted.
    fn sorted_chains(n: usize, chains: usize) -> Vec<usize> {
        let mut arr: Vec<usize> = (0..n).map(|i| i * 7919 % 1000).collect();
        for s in 0..chains {
            let mut chain: Vec<usize> = arr.iter().skip(s).step_by(chains).copied().collect();
            chain.sort();
            chain.into_iter().enumerate().for_each(|(k, value)| arr[s + k * chains] = value);
        }
        arr
    }

    #[test]
    fn a_tournament_merge_compares_ceil_log2_k_times_per_element() {
        for chains in 2..=40usize {
            let n = 20 * chains + 7;
            let mut arr = sorted_chains(n, chains);
            let mut aux = vec![0; TournamentMerge::aux_size(n)];
            let mut index = vec![0; TournamentMerge::index_size(chains)];
            let mut compares = Compares(0);
            TournamentMerge::merge(&mut arr, &mut aux, &mut index, 1, chains, 0, &mut compares);
            assert!(arr.windows(2).all(|pair| pair[0] <= pair[1]), "{chains} chains left unsorted");
            let rounds = chains.next_power_of_two().trailing_zeros() as usize;
            assert!(compares.0 <= n * rounds + chains, "{chains} chains took {} compares", compares.0);
        }
    }
}
