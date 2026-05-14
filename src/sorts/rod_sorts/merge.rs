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
//! - [`AuxMerge`]: a `branch`-way merge sort merge through an aux buffer —
//!   O(N · log(branch)) compares per pass, plus N reads / writes for the
//!   buffer round-trip. Asymptotically the same as merge sort overall.

use crate::traits::log_traits::SortLogger;

pub trait RodMerge {
    const NAME: &'static str;

    /// Merge the `branch` already-sorted sub-streams at stride `jump` into
    /// a single sorted stride-`jump` sequence over `arr`.
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        branch: usize,
        logger: &mut U,
    );
}

pub struct InsertionMerge;
impl RodMerge for InsertionMerge {
    const NAME: &'static str = "insertion";

    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        _branch: usize,
        logger: &mut U,
    ) {
        insertion_sort_jump(arr, jump, logger);
    }
}

pub struct AuxMerge;
impl RodMerge for AuxMerge {
    const NAME: &'static str = "aux";

    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        jump: usize,
        branch: usize,
        logger: &mut U,
    ) {
        // Number of elements at stride `jump`: positions 0, jump, 2·jump,
        // ... up to the last one < arr.len(). Matches the iteration range
        // of `insertion_sort_jump`.
        let virtual_len = arr.len().div_ceil(jump);
        if branch <= 1 || virtual_len <= 1 {
            return;
        }

        // Substream `s` holds virtual positions s, s + branch, s + 2·branch,
        // ... — i.e., physical positions (s + k·branch) · jump while in range.
        let stream_count = branch.min(virtual_len);
        let mut heads: Vec<usize> = (0..stream_count).collect();
        let mut aux = logger.create_aux_arr_t(virtual_len);

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
            logger.write_accross(arr, src_phys, &mut aux, write_idx);
            heads[min_s] += branch;
        }

        for write_idx in 0..virtual_len {
            let dst_phys = write_idx * jump;
            logger.write_accross(&aux, write_idx, arr, dst_phys);
        }

        logger.free_aux_arr_t(&aux);
    }
}

/// Strided insertion sort: sort `arr` viewed as a stride-`jump` virtual
/// array. Reused for the base case and the intermediate pass — those see
/// unsorted strided data, not merge structure, so they always run this.
pub(crate) fn insertion_sort_jump<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let mut i = 0;
    while i < arr.len() {
        let mut ii = i;
        while ii >= jump {
            if !logger.cond_swap_lt(arr, ii, ii - jump) {
                break;
            }
            ii -= jump;
        }
        i += jump;
    }
}
