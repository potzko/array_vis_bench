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
//! - [`TournamentMerge`]: the same buffer round trip with the chain heads in
//!   a loser tree, so at most ceil(log2 branch) compares per element per
//!   pass, and N log N whatever the strategy.
//! - [`TreeMerge`]: the same round trip with the chain heads in a search tree
//!   of the tree sort family, least out and next head in, as balanced as the
//!   tree it is given.
//!
//! ## Aux buffer ownership
//!
//! The top-level merge needs an output of size N (jump=1, virtual_len=N).
//! Every shallower merge needs strictly less. The recursion is sequential,
//! so one [`Scratch`] suffices for the entire sort and is allocated once at
//! the top, see [`super::rod_sort::RodSort::sort`]: the output and a key per
//! chain in one element buffer, the chain heads and a tree over them in one
//! index buffer, each section at a fixed place and shown as an array of its
//! own. [`InsertionMerge`] asks for none, so it allocates nothing.

use std::marker::PhantomData;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::HasSpace;
use shell_sort_lib::insert_into_chain;
use sort_logger::{split_sections, SortLogger};
use tree_sort_lib::{Layout, Nodes, Packed, Tree};

/// The lengths of a [`Scratch`]'s sections.
#[derive(Clone, Copy, Default)]
pub struct Sections {
    pub out: usize,
    pub keys: usize,
    pub heads: usize,
    pub tree: usize,
}

impl Sections {
    /// The element buffer's sections, the output then the keys.
    pub fn elems(self) -> [usize; 2] { [self.out, self.keys] }

    /// The index buffer's sections, the heads then the tree.
    pub fn words(self) -> [usize; 2] { [self.heads, self.tree] }
}

/// The scratch every merge of a sort shares: the output and a key per chain,
/// the chain heads and a tree over them. The driver creates it once as two
/// buffers in sections, so the visualiser shows each section as an array of
/// its own.
pub struct Scratch<'a, T> {
    pub out: &'a mut [T],
    pub keys: &'a mut [T],
    pub heads: &'a mut [usize],
    pub tree: &'a mut [usize],
}

impl<'a, T> Scratch<'a, T> {
    /// `elems` and `words` cut into the sections `sections` lays them out in.
    pub fn over(elems: &'a mut [T], words: &'a mut [usize], sections: Sections) -> Self {
        let [out, keys] = split_sections(elems, sections.elems());
        let [heads, tree] = split_sections(words, sections.words());
        Scratch { out, keys, heads, tree }
    }
}

pub trait RodMerge {
    const NAME: &'static str;

    /// The scratch the merges of a sort over `n` need, whose top level has
    /// `branch` chains, created once by the driver and shared by every merge.
    /// The top level's branch is the widest a sort meets while the strategy's
    /// branch grows with the length; a merge handed less brings its own. None
    /// for a merge that works in place.
    #[inline]
    fn sections(_n: usize, _branch: usize) -> Sections {
        Sections::default()
    }

    /// Merge the `branch` already-sorted sub-streams at stride `jump` into
    /// a single sorted stride-`jump` sequence over `arr`.
    ///
    /// `scratch` holds the sections [`Self::sections`] asked for, each empty
    /// when it asked for none. `inter` is the branching strategy's intermediate factor
    /// (already gated by the caller — 0 means skip). The InsertionMerge
    /// uses `inter` for an extra pre-pass at stride `jump * inter`; the
    /// AuxMerge ignores it (it would scramble the branch sub-streams).
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut Scratch<T>,
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
        _scratch: &mut Scratch<T>,
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
    fn sections(n: usize, branch: usize) -> Sections {
        Sections { out: n, heads: branch, ..Sections::default() }
    }

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut Scratch<T>,
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
        let chains = branch.min(virtual_len);
        with_scratch(scratch, Self::sections(virtual_len, chains), logger, |s, logger| {
            scan_merge(arr, s.out, s.heads, jump, branch, logger)
        });
    }
}

pub struct TournamentMerge;
impl RodMerge for TournamentMerge {
    const NAME: &'static str = "tournament";

    #[inline]
    fn sections(n: usize, branch: usize) -> Sections {
        Sections { out: n, heads: branch, tree: branch, ..Sections::default() }
    }

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut Scratch<T>,
        jump: usize,
        branch: usize,
        _inter: usize,
        logger: &mut U,
    ) {
        let virtual_len = arr.len().div_ceil(jump);
        if branch <= 1 || virtual_len <= 1 {
            return;
        }
        let chains = branch.min(virtual_len);
        with_scratch(scratch, Self::sections(virtual_len, chains), logger, |s, logger| match chains {
            2 => two_way_merge(arr, s.out, s.heads, jump, logger),
            _ => tournament_merge(arr, s.out, s.heads, s.tree, jump, branch, logger),
        });
    }
}

pub struct TreeMerge<B: Tree>(PhantomData<B>);
impl<B: Tree> RodMerge for TreeMerge<B> {
    const NAME: &'static str = "tree";

    #[inline]
    fn sections(n: usize, branch: usize) -> Sections {
        Sections { out: n, keys: branch, heads: branch, tree: Packed::words(branch) }
    }

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut Scratch<T>,
        jump: usize,
        branch: usize,
        _inter: usize,
        logger: &mut U,
    ) {
        let virtual_len = arr.len().div_ceil(jump);
        if branch <= 1 || virtual_len <= 1 {
            return;
        }
        let chains = branch.min(virtual_len);
        with_scratch(scratch, Self::sections(virtual_len, chains), logger, |s, logger| {
            tree_merge::<B, T, U>(arr, s, jump, branch, logger)
        });
    }
}

// Runs `body` on the front of each section of `scratch`, as much as `need`
// asks, or past the output on sections of its own when the driver's are
// shorter, which only a strategy whose branch narrows with the length can
// cause.
fn with_scratch<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    scratch: &mut Scratch<T>,
    need: Sections,
    logger: &mut U,
    body: impl FnOnce(&mut Scratch<T>, &mut U),
) {
    let out = &mut scratch.out[..need.out];
    if scratch.keys.len() >= need.keys && scratch.heads.len() >= need.heads && scratch.tree.len() >= need.tree {
        let (keys, heads, tree) = (&mut scratch.keys[..need.keys], &mut scratch.heads[..need.heads], &mut scratch.tree[..need.tree]);
        body(&mut Scratch { out, keys, heads, tree }, logger);
    } else {
        let own = Sections { out: 0, ..need };
        let mut elems = logger.create_aux_arr_split_t(&own.elems());
        let mut words = logger.create_aux_arr_split(&own.words());
        let Scratch { keys, heads, tree, .. } = Scratch::over(&mut elems, &mut words, own);
        body(&mut Scratch { out, keys, heads, tree }, logger);
        logger.free_aux_arr_split(&words, &own.words());
        logger.free_aux_arr_split_t(&elems, &own.elems());
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
fn tournament_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    aux: &mut [T],
    heads: &mut [usize],
    tree: &mut [usize],
    jump: usize,
    branch: usize,
    logger: &mut U,
) {
    let len = aux.len();
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

// Two chains need no tree, one compare per output either way. The heads live
// in locals and the index only mirrors them for the visualiser: `classic` at
// 2^20 took 227 ns per element through the tree, 185 with the heads read back
// from the index, and 139 like this.
fn two_way_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    aux: &mut [T],
    heads: &mut [usize],
    jump: usize,
    logger: &mut U,
) {
    let len = aux.len();
    let (mut even, mut odd) = (0, 1);
    logger.write_data_u(heads, 0, even);
    logger.write_data_u(heads, 1, odd);
    for write_idx in 0..len {
        if odd < len && (even >= len || logger.cmp_lt_accross(arr, odd * jump, arr, even * jump)) {
            logger.write_accross(arr, odd * jump, aux, write_idx);
            odd += 2;
            logger.write_data_u(heads, 1, odd);
        } else {
            logger.write_accross(arr, even * jump, aux, write_idx);
            even += 2;
            logger.write_data_u(heads, 0, even);
        }
    }
    for write_idx in 0..len {
        logger.write_accross(aux, write_idx, arr, write_idx * jump);
    }
}

// Merges the chains into `out` through a search tree of the tree family, then
// copies `out` back at stride `jump`. Node `s` is chain `s`, keyed by a copy of
// its head in `keys`: the least node comes out, and unless its chain is spent
// it goes back in under the chain's next head. The tree holds the nodes.
fn tree_merge<B: Tree, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    scratch: &mut Scratch<T>,
    jump: usize,
    branch: usize,
    logger: &mut U,
) {
    let Scratch { out, keys, heads, tree: meta } = scratch;
    let len = out.len();
    for s in 0..heads.len() {
        logger.write_data_u(heads, s, s);
        logger.write_accross(arr, s * jump, keys, s);
    }
    let mut nodes = Nodes::<T, U, Packed>::over(keys, meta, logger);
    for s in 0..heads.len() {
        nodes.attach(s, B::LEAF_RANK);
        B::inserted(&mut nodes, s);
    }
    for write_idx in 0..len {
        let (s, parent) = nodes.unlink_min();
        B::removed(&mut nodes, parent);
        let next = heads[s] + branch;
        nodes.logger().write_accross(arr, heads[s] * jump, out, write_idx);
        nodes.logger().write_data_u(heads, s, next);
        if next < len {
            nodes.set_key(s, arr[next * jump]);
            nodes.attach(s, B::LEAF_RANK);
            B::inserted(&mut nodes, s);
        }
    }
    for write_idx in 0..len {
        logger.write_accross(out, write_idx, arr, write_idx * jump);
    }
}

/// Strided insertion sort: sort `arr` viewed as a stride-`jump` virtual
/// array. Reused for the base case and the intermediate pass — those see
/// unsorted strided data, not merge structure, so they always run this.
/// Each step is the shell sorts' hold and shift insertion into the chain.
pub(crate) fn insertion_sort_jump<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (jump..arr.len()).step_by(jump) {
        insert_into_chain(arr, i, jump, logger);
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
impl<B: Tree> HasSpace for TreeMerge<B> {
    const SPACE: Complexity = Complexity::N1;
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::testing::Compares;
    use sort_logger::{SortLog, VisualizerLogger};
    use tree_sort_lib::{Avl, Naive};

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

    // One merge through `M` of `chains` chains of `arr`, returning its compares.
    fn merged<M: RodMerge>(arr: &mut [usize], chains: usize) -> usize {
        let sections = M::sections(arr.len(), chains);
        let mut elems = vec![0; sections.elems().iter().sum()];
        let mut words = vec![0; sections.words().iter().sum()];
        let mut compares = Compares(0);
        M::merge(arr, &mut Scratch::over(&mut elems, &mut words, sections), 1, chains, 0, &mut compares);
        assert!(arr.windows(2).all(|pair| pair[0] <= pair[1]), "{chains} chains left unsorted");
        compares.0
    }

    #[test]
    fn a_tournament_merge_compares_ceil_log2_k_times_per_element() {
        for chains in 2..=40usize {
            let n = 20 * chains + 7;
            let compares = merged::<TournamentMerge>(&mut sorted_chains(n, chains), chains);
            let rounds = chains.next_power_of_two().trailing_zeros() as usize;
            assert!(compares <= n * rounds + chains, "{chains} chains took {compares} compares");
        }
    }

    #[test]
    fn an_avl_tree_merge_compares_at_most_its_height_per_attach() {
        // Every output attaches one node, and an AVL tree of k nodes is under
        // 1.4405 log2(k + 2) levels tall.
        for chains in 2..=40usize {
            let n = 20 * chains + 7;
            let height = (1.4405 * ((chains + 2) as f64).log2()) as usize;
            let compares = merged::<TreeMerge<Avl<1>>>(&mut sorted_chains(n, chains), chains);
            assert!(compares <= n * height, "{chains} chains took {compares} compares");
        }
    }

    #[test]
    fn a_naive_tree_merge_sorts_and_degrades_to_a_scan_on_presorted_chains() {
        for chains in 2..=40usize {
            merged::<TreeMerge<Naive>>(&mut sorted_chains(20 * chains + 7, chains), chains);
        }
        // Presorted, every next head is past all the others, so the tree is a
        // right path and each attach walks it.
        let (n, chains) = (20 * 16 + 7, 16);
        let compares = merged::<TreeMerge<Naive>>(&mut (0..n).collect::<Vec<_>>(), chains);
        assert!(compares >= (n - chains) * (chains - 1), "{compares} compares");
    }
}
