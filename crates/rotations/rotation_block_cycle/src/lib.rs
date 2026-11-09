//! Block cycle rotation (Blomer and Bux, 2026). Gries-Mills swaps the smaller
//! side past the larger one block at a time; block cycle fuses every swap of
//! such a run into one cyclic shift of the blocks, moved a batch of adjacent
//! columns at a time through a small buffer, and finishes with a buffered
//! rotation once the smaller side fits that buffer.
//!
//! - `block_cycle`: the algorithm over a given buffer and batch width.
//! - `BlockCycleRotation`: the catalog rotation, the paper's 256 cell buffer
//!   and 32 byte batches.

use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{buf_rotate_left_using, buf_rotate_right_using, Rotation};

const BUFFER: usize = 256;
const BATCH_BYTES: usize = 32;

pub struct BlockCycleRotation;

impl Rotation for BlockCycleRotation {
    const NAME: &'static str = "block cycle";

    // The smaller side of `n` is at most `n / 2`, so more would never fill.
    #[inline]
    fn scratch_size(n: usize) -> usize {
        BUFFER.min(n / 2)
    }

    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let cells = scratch.len().min(BUFFER);
        let batch = (BATCH_BYTES / std::mem::size_of::<T>().max(1)).clamp(1, cells.max(1));
        block_cycle(arr, split_ind, &mut scratch[..cells], batch, logger);
    }
}

/// Rotate `arr` by `split`, cycling `batch` columns at a time through
/// `buffer`. Needs `1 <= batch <= buffer.len()` once both sides outgrow it.
///
/// With `k` the smaller side, the larger side holds `m` whole blocks of
/// length `k` next to it. Cycling the smaller block past all of them puts
/// those `m k` elements in place for `(m + 2) k` moves, where Gries-Mills
/// pays `3 m k` for the same `m` swaps, and leaves the smaller block against
/// the remainder of the larger side, a rotation of `k` and `k' < k`.
pub fn block_cycle<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    split: usize,
    buffer: &mut [T],
    batch: usize,
    logger: &mut U,
) {
    let (mut lo, mut mid, mut hi) = (0, split, arr.len());
    while lo < mid && mid < hi {
        let (left, right) = (mid - lo, hi - mid);
        if left.min(right) <= buffer.len() {
            if left <= right {
                buf_rotate_left_using(&mut arr[lo..hi], left, buffer, logger);
            } else {
                buf_rotate_right_using(&mut arr[lo..hi], left, buffer, logger);
            }
            return;
        }
        if left <= right {
            let blocks = right / left;
            cycle_blocks(arr, |ind| lo + ind * left, left, blocks, buffer, batch, logger);
            (lo, mid) = (lo + blocks * left, mid + blocks * left);
        } else {
            let blocks = left / right;
            cycle_blocks(arr, |ind| hi - right - ind * right, right, blocks, buffer, batch, logger);
            (mid, hi) = (mid - blocks * right, hi - blocks * right);
        }
    }
}

/// Move block `ind + 1` into block `ind` for every `ind < blocks`, and block
/// 0 into block `blocks`, where block `ind` is the `len` elements from
/// `start(ind)`. Each pass carries `batch` adjacent columns of every block,
/// so it reads and writes runs of `batch` cells rather than single ones.
fn cycle_blocks<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: impl Fn(usize) -> usize,
    len: usize,
    blocks: usize,
    buffer: &mut [T],
    batch: usize,
    logger: &mut U,
) {
    for col in (0..len).step_by(batch) {
        let width = batch.min(len - col);
        logger.copy_range(arr, start(0) + col, buffer, 0, width);
        for ind in 0..blocks {
            let (dst, src) = (start(ind) + col, start(ind + 1) + col);
            for i in 0..width { logger.write(arr, dst + i, src + i); }
        }
        logger.copy_range(buffer, 0, arr, start(blocks) + col, width);
    }
}

impl array_vis_bench_traits::HasSpace for BlockCycleRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}

impl array_vis_bench_traits::HasTimeBounds for BlockCycleRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for BlockCycleRotation {
    const STABLE: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sort_logger::{NoOpLogger, SortLog, VisualizerLogger};

    fn logger() -> VisualizerLogger<u32> {
        VisualizerLogger { type_ghost: std::marker::PhantomData, log: Vec::new() }
    }

    fn rotated(n: usize, split: usize) -> Vec<u32> {
        let mut want: Vec<u32> = (0..n as u32).collect();
        want.rotate_left(split);
        want
    }

    // One assignment is one move, a swap three.
    fn moves(log: &VisualizerLogger<u32>) -> usize {
        log.log.iter().map(|e| match e {
            SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => 1,
            SortLog::Swap { .. } => 3,
            _ => 0,
        }).sum()
    }

    // The paper's m(n, k, b): `n + k` once the smaller side `k` fits the
    // buffer, otherwise one cycle of `n / k - 1` blocks of the larger side
    // and the smaller block, `(n / k + 1) k` moves, then the remainder.
    fn paper_moves(n: usize, k: usize, b: usize) -> usize {
        let k = k.min(n - k);
        match k {
            0 => 0,
            _ if k <= b => n + k,
            _ => (n / k + 1) * k + paper_moves(n - (n / k - 1) * k, n - n / k * k, b),
        }
    }

    #[test]
    fn every_split_rotates_for_every_buffer_and_batch() {
        for n in 0..48 {
            for split in 0..=n {
                for cells in 1..=6 {
                    for batch in 1..=cells {
                        let mut arr: Vec<u32> = (0..n as u32).collect();
                        block_cycle(&mut arr, split, &mut vec![0; cells], batch, &mut NoOpLogger);
                        assert_eq!(arr, rotated(n, split), "n={n} split={split} cells={cells} batch={batch}");
                    }
                }
            }
        }
    }

    #[test]
    fn moves_are_the_papers_count() {
        for n in 0..64 {
            for split in 0..=n {
                for cells in 1..=5 {
                    let mut arr: Vec<u32> = (0..n as u32).collect();
                    let mut log = logger();
                    block_cycle(&mut arr, split, &mut vec![0; cells], cells.div_ceil(2), &mut log);
                    assert_eq!(moves(&log), paper_moves(n, split, cells), "n={n} split={split} cells={cells}");
                }
            }
        }
    }

    #[test]
    fn the_catalog_rotation_cycles_past_its_buffer_without_allocating() {
        for n in [0, 1, 2, 3, 100, 511, 512, 513, 514, 515, 600, 1031] {
            for split in 0..=n {
                let mut arr: Vec<u32> = (0..n as u32).collect();
                let mut scratch = vec![0; BlockCycleRotation::scratch_size(n)];
                let mut log = logger();
                BlockCycleRotation::rotate(&mut arr, split, &mut scratch, &mut log);
                assert_eq!(arr, rotated(n, split), "n={n} split={split}");
                assert!(!log.log.iter().any(|e| matches!(e, SortLog::CreateAuxArrT { .. })));
            }
        }
    }
}
