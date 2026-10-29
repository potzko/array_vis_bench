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
//! once at the top — see [`super::rod_sort::RodSort::sort`]. The
//! `NEEDS_AUX` flag lets [`InsertionMerge`] skip the allocation entirely.

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::HasSpace;
use sort_logger::SortLogger;

pub trait RodMerge {
    const NAME: &'static str;

    /// Whether this merge needs an aux buffer of size N. When `false`, the
    /// caller may pass an empty slice and skip allocation entirely.
    const NEEDS_AUX: bool;

    /// Merge the `branch` already-sorted sub-streams at stride `jump` into
    /// a single sorted stride-`jump` sequence over `arr`.
    ///
    /// `aux` is a pre-allocated scratch buffer of size ≥
    /// `arr.len().div_ceil(jump)` when `NEEDS_AUX` is true; otherwise may
    /// be empty. `inter` is the branching strategy's intermediate factor
    /// (already gated by the caller — 0 means skip). The InsertionMerge
    /// uses `inter` for an extra pre-pass at stride `jump * inter`; the
    /// AuxMerge ignores it (it would scramble the branch sub-streams).
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        aux: &mut [T],
        jump: usize,
        branch: usize,
        inter: usize,
        logger: &mut U,
    );
}

pub struct InsertionMerge;
impl RodMerge for InsertionMerge {
    const NAME: &'static str = "insertion";
    const NEEDS_AUX: bool = false;

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _aux: &mut [T],
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
    const NEEDS_AUX: bool = true;

    #[inline]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        aux: &mut [T],
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

        // Use only the prefix we need; the caller's aux is the full
        // top-level allocation (size N), shared across the recursion.
        let aux = &mut aux[..virtual_len];

        // Substream `s` holds virtual positions s, s + branch, s + 2·branch,
        // ... — i.e., physical positions (s + k·branch) · jump while in range.
        let stream_count = branch.min(virtual_len);
        let mut heads: Vec<usize> = (0..stream_count).collect();

        for write_idx in 0..virtual_len {
            // Linear scan over the (small) number of active sub-streams.
            // For typical `branch` values (2–32) this beats a heap.
            let mut min_s = usize::MAX;
            for s in 0..stream_count {
                if heads[s] >= virtual_len {
                    continue;
                }
                min_s = if min_s == usize::MAX {
                    s
                } else {
                    let s_phys = heads[s] * jump;
                    let min_phys = heads[min_s] * jump;
                    if logger.cmp_lt_accross(arr, s_phys, arr, min_phys) {
                        s
                    } else {
                        min_s
                    }
                };
            }
            let src_phys = heads[min_s] * jump;
            logger.write_accross(arr, src_phys, aux, write_idx);
            heads[min_s] += branch;
        }

        for write_idx in 0..virtual_len {
            let dst_phys = write_idx * jump;
            logger.write_accross(aux, write_idx, arr, dst_phys);
        }
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
