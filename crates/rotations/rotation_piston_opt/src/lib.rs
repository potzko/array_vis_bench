//! Optimized piston rotation: Stew Forster's triple shift rotate (2025),
//! reimplemented from a description of it. Piston swaps the smaller side with
//! the window of its length at the far end of the larger side and puts only
//! that window in place. Here every column also passes through the cells
//! between the two windows, run as a ring, so the near window lands in place
//! too: four moves a column put two elements where piston's three put one.
//!
//! - `optimized_piston`: the algorithm over a given buffer.
//! - `OptimizedPistonRotation`: the catalog rotation, with a 256 cell buffer.

use sort_logger::{RegionKind, SortLogger};
use array_vis_bench_traits::role::rotation::{buf_rotate_left_using, forward_block_swap, Rotation};

// A pass moves runs of at most the smaller side or the ring, whichever is
// shorter, and short runs move an element at a time. The original finishes
// through a 1024 byte stack buffer once either fits it, 256 four byte cells.
const BUFFER: usize = 256;

pub struct OptimizedPistonRotation;

impl Rotation for OptimizedPistonRotation {
    const NAME: &'static str = "optimized piston";

    // A range never needs more than its own length.
    #[inline]
    fn scratch_size(n: usize) -> usize {
        BUFFER.min(n)
    }

    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let cells = scratch.len().min(BUFFER);
        optimized_piston(arr, split_ind, &mut scratch[..cells], logger);
    }
}

/// Rotate `arr` by `split`, finishing through `buffer` the first range whose
/// smaller side or difference of sides fits it. With an empty buffer every
/// pass runs in place, `2n - g` moves when `n / g` is even and `2n - 2g` when
/// it is odd, for `g` the gcd of the sides.
pub fn optimized_piston<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    split: usize,
    buffer: &mut [T],
    logger: &mut U,
) {
    let (mut lo, mut mid, mut hi) = (0, split, arr.len());
    while lo < mid && mid < hi {
        let (a, b) = (mid - lo, hi - mid);
        if a == b {
            forward_block_swap(arr, lo, mid, a, logger);
            return;
        }
        if a.min(b) <= buffer.len() || a.abs_diff(b) <= buffer.len() {
            finish(&mut arr[lo..hi], a, buffer, logger);
            return;
        }
        (lo, mid, hi) = pass(arr, lo, mid, hi, logger);
    }
}

/// One pass over `[lo, hi)` split at `mid`, returning the next range. With
/// `s` the smaller side, the `s` cells at each end receive their final
/// contents and the cells between them, the ring, keep the rest rotated by
/// `s` modulo the ring's length.
///
/// Column `j` moves the near window's element into the far window, the far
/// window's into ring slot `j` modulo the ring, and what that slot held into
/// the near window. Each slot is refilled once a lap of the ring's length, so
/// a far element waits exactly the distance it sits from its place, and the
/// near window always receives its own. The near window is the smaller side,
/// at the left end when that side is, and laps then run from the left ends;
/// otherwise they run from the right.
///
/// The columns of one lap touch three disjoint runs, so a lap may run in any
/// order, and every lap runs ascending: LLVM vectorises none of the
/// descending loops here. A hundred thousand `u32` split at `n - n / 5`, two
/// right smaller passes, took 0.26 ns per element with laps descending and
/// 0.063 ascending, `u64` 0.28 and 0.13; over all splits 0.20 and 0.079,
/// `u64` 0.25 and 0.15.
fn pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) -> (usize, usize, usize) {
    let up = mid - lo < hi - mid;
    let (s, ring) = ((mid - lo).min(hi - mid), (mid - lo).abs_diff(hi - mid));
    let (at, end) = (lo + s, hi - s);
    let used = ring.min(s);
    let held = if up { at } else { end - used };
    logger.hold_region(arr, held, used, RegionKind::Buffer);
    let (mut j, mut w) = (0, 0);
    while j < s {
        w = ring.min(s - j);
        let back = s - j - w;
        if up { cycle(arr, lo + j, at, end + j, w, logger) } else { cycle(arr, end + back, end - w, lo + back, w, logger) }
        j += w;
    }
    logger.release_region(arr, held, used, RegionKind::Buffer);
    let turn = if w == ring { 0 } else { w };
    if up { (at, at + turn, end) } else { (at, end - turn, end) }
}

/// Over `w` columns of the runs at `near`, `ring` and `far`, the near run
/// receives the ring's elements, the ring the far ones and the far run the
/// near ones.
fn cycle<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], near: usize, ring: usize, far: usize, w: usize, logger: &mut U) {
    for i in 0..w {
        let t = arr[near + i];
        logger.write(arr, near + i, ring + i);
        logger.write(arr, ring + i, far + i);
        logger.write_data(arr, far + i, t);
    }
}

/// Rotate `arr` by `a` through `buffer`: the smaller side through the buffer
/// when it fits, `n + s` moves, else a bridge that buffers only the
/// difference `o` of the sides, `n + o` moves.
fn finish<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], a: usize, buffer: &mut [T], logger: &mut U) {
    let n = arr.len();
    let b = n - a;
    if a < b && a <= buffer.len() {
        buf_rotate_left_using(arr, a, buffer, logger);
    } else if b < a && b <= buffer.len() {
        logger.copy_range(arr, a, buffer, 0, b);
        shift_up(arr, a, b, logger);
        logger.copy_range(buffer, 0, arr, 0, b);
    } else if a < b {
        let o = b - a;
        logger.copy_range(arr, n - o, buffer, 0, o);
        bridge_down(arr, a, b, logger);
        logger.copy_range(buffer, 0, arr, a, o);
    } else {
        // Step `i` moves a left element up by `b` and a right one down by
        // `a`, into cells the buffer emptied or an earlier step read.
        let o = a - b;
        logger.copy_range(arr, b, buffer, 0, o);
        for i in 0..b {
            logger.write(arr, b + i, i);
            logger.write(arr, i, a + i);
        }
        logger.copy_range(buffer, 0, arr, 2 * b, o);
    }
}

// Moving part of a range up an element at a time has to run from the top
// down, and LLVM vectorises no descending loop here. So such a move runs in
// pieces from the top, each ascending: as long as the distance moved when
// that is `CHUNK` or more, so a piece never overwrites its own source, else
// `BLOCK` cells read whole before any is written. A thousand `u32` over all
// splits took 0.152 ns per element moved an element at a time and 0.123 in
// pieces, `u64` 0.168 and 0.138; split one short of the middle, a bridge two
// cells wide, `u32` took 0.217 and 0.120, `u64` 0.217 and 0.145. Blocks
// are indexed where clippy asks for an iterator, which took that bridge 0.204.
const CHUNK: usize = 32;
const BLOCK: usize = 8;

/// Move `arr[..len]` up by `by`.
#[allow(clippy::needless_range_loop)]
fn shift_up<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], len: usize, by: usize, logger: &mut U) {
    let mut x = len;
    if by >= CHUNK {
        while x > 0 {
            let w = by.min(x);
            x -= w;
            for i in x..x + w { logger.write(arr, i + by, i); }
        }
        return;
    }
    while x >= BLOCK {
        x -= BLOCK;
        let block: [T; BLOCK] = arr[x..x + BLOCK].try_into().unwrap();
        // One check of the whole run spares the writes theirs.
        let _ = &arr[x + by..x + by + BLOCK];
        for i in 0..BLOCK { logger.write_data(arr, x + by + i, block[i]); }
    }
    for i in (0..x).rev() { logger.write(arr, i + by, i); }
}

/// The bridge with the left side smaller: step `i` moves `arr[i]` up by `b`
/// and `arr[a + i]` down to `i`. The cell step `i` fills is the one step
/// `i + b - a` reads, so the steps run from the top.
#[allow(clippy::needless_range_loop)]
fn bridge_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], a: usize, b: usize, logger: &mut U) {
    let (o, mut x) = (b - a, a);
    if o >= CHUNK {
        while x > 0 {
            let w = o.min(x);
            x -= w;
            for i in x..x + w {
                logger.write(arr, b + i, i);
                logger.write(arr, i, a + i);
            }
        }
        return;
    }
    while x >= BLOCK {
        x -= BLOCK;
        let lefts: [T; BLOCK] = arr[x..x + BLOCK].try_into().unwrap();
        let rights: [T; BLOCK] = arr[a + x..a + x + BLOCK].try_into().unwrap();
        let _ = &arr[b + x..b + x + BLOCK];
        for i in 0..BLOCK { logger.write_data(arr, b + x + i, lefts[i]); }
        for i in 0..BLOCK { logger.write_data(arr, x + i, rights[i]); }
    }
    for i in (0..x).rev() {
        logger.write(arr, b + i, i);
        logger.write(arr, i, a + i);
    }
}

impl array_vis_bench_traits::HasSpace for OptimizedPistonRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}

impl array_vis_bench_traits::HasTimeBounds for OptimizedPistonRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for OptimizedPistonRotation {
    const STABLE: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use array_vis_bench_traits::role::rotation::gcd;
    use sort_logger::{NoOpLogger, SortLog, VisualizerLogger};

    fn logger() -> VisualizerLogger<u32> {
        VisualizerLogger { type_ghost: std::marker::PhantomData, log: Vec::new() }
    }

    fn rotated(n: usize, split: usize) -> Vec<u32> {
        let mut want: Vec<u32> = (0..n as u32).collect();
        want.rotate_left(split);
        want
    }

    // The left side holds `b..n` and the right side `0..b`, so every value is
    // its own final index, as in the description's traces.
    fn labelled(a: usize, b: usize) -> Vec<u32> {
        (b..a + b).chain(0..b).map(|v| v as u32).collect()
    }

    // One assignment is one move: a write is one and a `Swap` three. With the
    // buffer off every `WriteData` is two, since it stores the element its
    // column first assigned to a temporary.
    fn assignments(log: &VisualizerLogger<u32>) -> usize {
        log.log.iter().map(|e| match e {
            SortLog::WriteInArr { .. } => 1,
            SortLog::WriteData { .. } => 2,
            SortLog::Swap { .. } => 3,
            _ => 0,
        }).sum()
    }

    #[test]
    fn every_split_rotates_with_and_without_a_buffer() {
        for n in 0..100 {
            for split in 0..=n {
                for cells in [0, 1, 2, 3, 7, 64] {
                    let mut arr: Vec<u32> = (0..n as u32).collect();
                    optimized_piston(&mut arr, split, &mut vec![0; cells], &mut NoOpLogger);
                    assert_eq!(arr, rotated(n, split), "n={n} split={split} cells={cells}");
                }
            }
        }
    }

    #[test]
    fn in_place_it_moves_two_n_less_the_gcd_once_or_twice() {
        for n in 1..120 {
            for split in 0..=n {
                let g = gcd(split, n - split);
                let want = if (n / g) % 2 == 0 { 2 * n - g } else { 2 * n - 2 * g };
                let mut arr: Vec<u32> = (0..n as u32).collect();
                let mut log = logger();
                optimized_piston(&mut arr, split, &mut [], &mut log);
                assert_eq!(assignments(&log), want, "n={n} split={split}");
            }
        }
    }

    // The description's worked examples: the array and the next range after
    // the first pass.
    #[test]
    fn first_passes_match_the_worked_examples() {
        let cases: [(usize, usize, Vec<u32>, (usize, usize, usize)); 4] = [
            (8, 13, [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 8, 9].into_iter().chain(13..21).collect(), (8, 11, 13)),
            (13, 8, [0, 1, 2, 3, 4, 5, 6, 7, 11, 12, 8, 9, 10].into_iter().chain(13..21).collect(), (8, 10, 13)),
            (4, 11, vec![0, 1, 2, 3, 7, 8, 9, 10, 4, 5, 6, 11, 12, 13, 14], (4, 8, 11)),
            (9, 4, vec![0, 1, 2, 3, 8, 4, 5, 6, 7, 9, 10, 11, 12], (4, 5, 9)),
        ];
        for (a, b, want, next) in cases {
            let mut arr = labelled(a, b);
            assert_eq!(pass(&mut arr, 0, a, a + b, &mut NoOpLogger), next, "({a}, {b})");
            assert_eq!(arr, want, "({a}, {b})");
        }
    }

    #[test]
    fn the_catalog_rotation_finishes_in_its_scratch_without_allocating() {
        for n in [0, 1, 2, 3, 100, 256, 257, 513, 771, 772, 1031] {
            for split in 0..=n {
                let mut arr: Vec<u32> = (0..n as u32).collect();
                let mut scratch = vec![0; OptimizedPistonRotation::scratch_size(n)];
                let mut log = logger();
                OptimizedPistonRotation::rotate(&mut arr, split, &mut scratch, &mut log);
                assert_eq!(arr, rotated(n, split), "n={n} split={split}");
                assert!(!log.log.iter().any(|e| matches!(e, SortLog::CreateAuxArrT { .. })));
            }
        }
    }
}
