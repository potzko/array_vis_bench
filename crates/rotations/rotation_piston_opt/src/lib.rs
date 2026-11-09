//! Optimized piston rotation: Stew Forster's triple shift rotate (2025),
//! reimplemented from a description of it. Piston swaps the smaller side with
//! the window of its length at the far end of the larger side and puts only
//! that window in place. Here every column also passes through the cells
//! between the two windows, run as a ring, so the near window lands in place
//! too: four moves a column put two elements where piston's three put one.
//!
//! - `optimized_piston`: the algorithm over a given buffer, cycling two or
//!   three windows a pass.
//! - `OptimizedPiston`: the catalog rotation, over its windows and buffer.
//! - `OptimizedPistonRotation`: two windows and a 256 cell buffer.

use sort_logger::{RegionKind, SortLogger};
use array_vis_bench_traits::role::rotation::{buf_rotate_left_using, forward_block_swap, Rotation};

/// The optimized piston cycling `WINDOWS` whole windows a pass, finishing
/// through a buffer of `BUFFER` cells, none for a rotation fully in place.
pub struct OptimizedPiston<const WINDOWS: usize, const BUFFER: usize>;

// A pass moves runs of at most the smaller side or the ring, whichever is
// shorter, and short runs move an element at a time. The original finishes
// through a 1024 byte stack buffer once either fits it, 256 four byte cells.
// Over all splits of a thousand `u32`, 64 cells took 0.139 ns per element,
// 128 took 0.133 and 256 took 0.120. 512 took 0.088, but a range of a
// thousand then always fits and never runs in place. From ten thousand on
// the size made no difference.
pub type OptimizedPistonRotation = OptimizedPiston<2, 256>;

impl<const WINDOWS: usize, const BUFFER: usize> Rotation for OptimizedPiston<WINDOWS, BUFFER> {
    const NAME: &'static str = match (WINDOWS, BUFFER) {
        (2, 0) => "optimized piston<in place>",
        (2, _) => "optimized piston",
        (_, 0) => "optimized piston<3 windows, in place>",
        _ => "optimized piston<3 windows>",
    };

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
        optimized_piston::<T, U, WINDOWS>(arr, split_ind, &mut scratch[..cells], logger);
    }
}

/// Rotate `arr` by `split`, cycling `WINDOWS` whole windows a pass, and
/// finishing through `buffer` the first range whose smaller side or
/// difference of sides fits it. With an empty buffer every pass runs in
/// place, and two windows take `2n - g` moves when `n / g` is even and
/// `2n - 2g` when it is odd, for `g` the gcd of the sides.
pub fn optimized_piston<T: Ord + Copy, U: ?Sized + SortLogger<T>, const WINDOWS: usize>(
    arr: &mut [T],
    split: usize,
    buffer: &mut [T],
    logger: &mut U,
) {
    const { assert!(WINDOWS == 2 || WINDOWS == 3, "a pass cycles two or three windows") };
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
        (lo, mid, hi) = pass::<T, U, WINDOWS>(arr, lo, mid, hi, buffer.len(), logger);
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
/// With three windows, and room for it, the larger side's `s` cells next to
/// the near window join the cycle between it and the ring and land in place
/// too: five moves a column for three elements. A ring of at most `cells`
/// would run laps too short to pay, so the pass leaves those cells to the
/// buffer. With two windows the caller has finished any such range already.
///
/// The columns of one lap touch disjoint runs, so a lap may run in any
/// order, and every lap runs ascending: LLVM vectorises none of the
/// descending loops here. A hundred thousand `u32` split at `n - n / 5`, two
/// right smaller passes, took 0.26 ns per element with laps descending and
/// 0.063 ascending, `u64` 0.28 and 0.13; over all splits 0.20 and 0.079,
/// `u64` 0.25 and 0.15.
fn pass<T: Ord + Copy, U: ?Sized + SortLogger<T>, const WINDOWS: usize>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    cells: usize,
    logger: &mut U,
) -> (usize, usize, usize) {
    let up = mid - lo < hi - mid;
    let (s, o) = ((mid - lo).min(hi - mid), (mid - lo).abs_diff(hi - mid));
    let next = if WINDOWS == 3 && o >= s { s } else { 0 };
    let ring = if o - next > cells { o - next } else { 0 };
    let (near, far) = if up { (lo, hi - s) } else { (hi - s, lo) };
    let beside = if up { near + s } else { near - s };
    let (at, end) = if up { (lo + s + next, hi - s) } else { (lo + s, hi - s - next) };
    if ring == 0 {
        cycle(arr, near, beside, far, s, logger);
        return if up { (at - s, at, end) } else { (at, end, end + s) };
    }
    let used = ring.min(s);
    let held = if up { at } else { end - used };
    logger.hold_region(arr, held, used, RegionKind::Buffer);
    let (mut j, mut w) = (0, 0);
    while j < s {
        w = ring.min(s - j);
        let (col, slot) = if up { (j, at) } else { (s - j - w, end - w) };
        match next {
            0 => cycle(arr, near + col, slot, far + col, w, logger),
            _ => cycle4(arr, near + col, beside + col, slot, far + col, w, logger),
        }
        j += w;
    }
    logger.release_region(arr, held, used, RegionKind::Buffer);
    let turn = if w == ring { 0 } else { w };
    if up { (at, at + turn, end) } else { (at, end - turn, end) }
}

/// Over `w` columns of the runs at `x`, `y` and `z`, each run receives the
/// next one's elements and `z` receives those of `x`.
fn cycle<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], x: usize, y: usize, z: usize, w: usize, logger: &mut U) {
    for i in 0..w {
        let t = arr[x + i];
        logger.write(arr, x + i, y + i);
        logger.write(arr, y + i, z + i);
        logger.write_data(arr, z + i, t);
    }
}

/// `cycle` over four runs.
fn cycle4<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], x: usize, y: usize, z: usize, v: usize, w: usize, logger: &mut U) {
    for i in 0..w {
        let t = arr[x + i];
        logger.write(arr, x + i, y + i);
        logger.write(arr, y + i, z + i);
        logger.write(arr, z + i, v + i);
        logger.write_data(arr, v + i, t);
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

impl<const WINDOWS: usize, const BUFFER: usize> array_vis_bench_traits::HasSpace for OptimizedPiston<WINDOWS, BUFFER> {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}

impl<const WINDOWS: usize, const BUFFER: usize> array_vis_bench_traits::HasTimeBounds for OptimizedPiston<WINDOWS, BUFFER> {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl<const WINDOWS: usize, const BUFFER: usize> array_vis_bench_traits::HasStability for OptimizedPiston<WINDOWS, BUFFER> {
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

    fn rotates_every_split<const WINDOWS: usize>() {
        for n in 0..100 {
            for split in 0..=n {
                for cells in [0, 1, 2, 3, 7, 64] {
                    let mut arr: Vec<u32> = (0..n as u32).collect();
                    optimized_piston::<_, _, WINDOWS>(&mut arr, split, &mut vec![0; cells], &mut NoOpLogger);
                    assert_eq!(arr, rotated(n, split), "windows={WINDOWS} n={n} split={split} cells={cells}");
                }
            }
        }
    }

    #[test]
    fn every_split_rotates_with_and_without_a_buffer() {
        rotates_every_split::<2>();
        rotates_every_split::<3>();
    }

    #[test]
    fn in_place_it_moves_two_n_less_the_gcd_once_or_twice() {
        for n in 1..120 {
            for split in 0..=n {
                let g = gcd(split, n - split);
                let want = if (n / g).is_multiple_of(2) { 2 * n - g } else { 2 * n - 2 * g };
                let mut arr: Vec<u32> = (0..n as u32).collect();
                let mut log = logger();
                optimized_piston::<_, _, 2>(&mut arr, split, &mut [], &mut log);
                assert_eq!(assignments(&log), want, "n={n} split={split}");
            }
        }
    }

    // The description simulates three windows and the ring at 1.78n moves
    // over all splits of a thousand, never more than 2n.
    #[test]
    fn three_windows_in_place_average_one_point_seven_eight_n() {
        let moves = |n: usize, split: usize| {
            let mut arr: Vec<u32> = (0..n as u32).collect();
            let mut log = logger();
            optimized_piston::<_, _, 3>(&mut arr, split, &mut [], &mut log);
            assignments(&log)
        };
        for n in 1..120 {
            for split in 0..=n { assert!(moves(n, split) <= 2 * n, "n={n} split={split}"); }
        }
        let total: usize = (1..1000).map(|split| moves(1000, split)).sum();
        assert_eq!((total as f64 / 999.0 / 10.0).round(), 178.0);
    }

    // The description's worked examples: the array and the next range after
    // the first pass.
    #[test]
    fn first_passes_match_the_worked_examples() {
        let cases = [
            (8, 13, [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 8, 9].into_iter().chain(13..21).collect::<Vec<u32>>(), (8, 11, 13)),
            (13, 8, [0, 1, 2, 3, 4, 5, 6, 7, 11, 12, 8, 9, 10].into_iter().chain(13..21).collect(), (8, 10, 13)),
            (4, 11, vec![0, 1, 2, 3, 7, 8, 9, 10, 4, 5, 6, 11, 12, 13, 14], (4, 8, 11)),
            (9, 4, vec![0, 1, 2, 3, 8, 4, 5, 6, 7, 9, 10, 11, 12], (4, 5, 9)),
        ];
        for (a, b, want, next) in cases {
            let mut arr = labelled(a, b);
            assert_eq!(pass::<_, _, 2>(&mut arr, 0, a, a + b, 0, &mut NoOpLogger), next, "({a}, {b})");
            assert_eq!(arr, want, "({a}, {b})");
        }
    }

    fn finishes_in_its_scratch<R: Rotation>() {
        for n in [0, 1, 2, 3, 100, 256, 257, 513, 771, 772, 1031] {
            for split in 0..=n {
                let mut arr: Vec<u32> = (0..n as u32).collect();
                let mut scratch = vec![0; R::scratch_size(n)];
                let mut log = logger();
                R::rotate(&mut arr, split, &mut scratch, &mut log);
                assert_eq!(arr, rotated(n, split), "{} n={n} split={split}", R::NAME);
                assert!(!log.log.iter().any(|e| matches!(e, SortLog::CreateAuxArrT { .. })), "{}", R::NAME);
            }
        }
    }

    #[test]
    fn the_catalog_rotations_finish_in_their_scratch_without_allocating() {
        finishes_in_its_scratch::<OptimizedPistonRotation>();
        finishes_in_its_scratch::<OptimizedPiston<2, 0>>();
        finishes_in_its_scratch::<OptimizedPiston<3, 256>>();
    }
}
