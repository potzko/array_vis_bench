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
        if index.len() < Self::index_size(chains) {
            let mut heads = logger.create_aux_arr(chains);
            scan_merge(arr, aux, &mut heads, jump, branch, logger);
            logger.free_aux_arr(&heads);
        } else {
            scan_merge(arr, aux, &mut index[..chains], jump, branch, logger);
        }
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
}
