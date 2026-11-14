//! The `StablePartition` fillers. `RotationPartition` halves the region,
//! partitions each half and rotates the middle, in one buffered pass when the
//! scratch holds the region. `SlidingPartition` gathers the matches in one
//! block that rides behind the scan, as the scan extraction gathers its keys.
//! `BlockPartition` works through a workspace it is lent: the scan leaves
//! whole blocks of matches and of the rest, and the blocks are then put in
//! order, one move each, by the marks that say which kind each was.

use array_vis_bench_traits::role::keys::rotate_region;
use array_vis_bench_traits::role::{Marks, Rotation, StablePartition, Workspace};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

/// Divide and conquer over the rotation: each half partitioned, then one
/// rotation brings the right half's matches up against the left half's. One
/// comparison per element and `O(n log n)` moves, `O(n log (n / s))` with `s`
/// cells of scratch, since a region the scratch holds is partitioned in one pass.
pub struct RotationPartition;
impl StablePartition for RotationPartition {
    const NAME: &'static str = "rotation";
    const MOVES: Complexity = Complexity::N_LOG_N;
    const MOVES_BUFFERED: Complexity = Complexity::N1;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_rotation::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_rotation::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

// `matches` says whether the element at an index is one of those to bring to
// the front, by one logged comparison against the pivot.
fn by_rotation<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let n = hi - lo;
    if n == 0 { return lo; }
    if n == 1 { return lo + usize::from(matches(log, arr, lo)); }
    if n <= scratch.len() { return buffered(arr, lo, hi, matches, scratch, log); }
    let mid = lo + n / 2;
    let left = by_rotation::<R, T, U, F>(arr, lo, mid, matches, scratch, log);
    let right = by_rotation::<R, T, U, F>(arr, mid, hi, matches, scratch, log);
    rotate_region::<R, T, U>(arr, left, mid, right, scratch, log);
    left + (right - mid)
}

// One pass from the right: a match goes out to the scratch and an element that
// stays shifts right by the matches passed, so the matches' slots open at the
// front and the scratch empties into them in order. `n + m` moves for `m` matches.
fn buffered<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let mut out = 0;
    for i in (lo..hi).rev() {
        if matches(log, arr, i) {
            log.write_accross(arr, i, scratch, out);
            out += 1;
        } else if out > 0 {
            log.write(arr, i + out, i);
        }
    }
    for k in 0..out { log.write_accross(scratch, out - 1 - k, arr, lo + k); }
    lo + out
}

/// The matches gather in one block that rides behind the scan: a run of
/// elements that stay costs one rotation of the block past it, so the moves are
/// `O(n + m r)` for `m` matches in `r` runs, linear when the matches are few or
/// clustered and quadratic at worst. One comparison per element.
pub struct SlidingPartition;
impl StablePartition for SlidingPartition {
    const NAME: &'static str = "sliding";
    const MOVES: Complexity = Complexity::N_SQUARED;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_sliding::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_sliding::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

fn by_sliding<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    // The block is arr[first..first + found], the scan is at i.
    let (mut first, mut found) = (lo, 0);
    for i in lo..hi {
        if !matches(log, arr, i) { continue; }
        rotate_region::<R, T, U>(arr, first, first + found, i, scratch, log);
        first = i - found;
        found += 1;
    }
    rotate_region::<R, T, U>(arr, lo, first, first + found, scratch, log);
    lo + found
}

/// Linear through a lent workspace of the square root of the region or more.
/// The scan streams the matches into one half of the buffer and the rest into
/// the other, and a half that fills is written back as a block into the cells
/// the scan has emptied, so the region becomes whole blocks of one kind each,
/// in the order they filled. A mark per block records its kind, from which the
/// place of every block follows, and the blocks are moved there along the
/// cycles of that permutation, a second mark per block saying it is placed:
/// one move per element and a read of every mark per block. Two marks per
/// block is what bounds the region, a quarter of the buffer's square; a longer
/// one is halved and rotated, and without a workspace it is the rotation
/// partition.
pub struct BlockPartition;
impl StablePartition for BlockPartition {
    const NAME: &'static str = "block";
    const MOVES: Complexity = Complexity::N_LOG_N;
    const MOVES_BUFFERED: Complexity = Complexity::N1;
    const MOVES_LENT: Complexity = Complexity::N1;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }

    fn partition_lent<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        work: &mut Workspace<M, T>,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_blocks::<M, R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), work, scratch, log)
    }

    fn partition_at_most_lent<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        work: &mut Workspace<M, T>,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_blocks::<M, R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), work, scratch, log)
    }
}

// A buffer shorter than this is not worth cutting into blocks.
const MIN_BUFFER: usize = 16;

#[allow(clippy::too_many_arguments)]
fn by_blocks<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    work: &mut Workspace<M, T>,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let n = hi - lo;
    let block = work.buffer.len() / 2;
    if n < 2 || n <= scratch.len() || work.buffer.len() < MIN_BUFFER { return by_rotation::<R, T, U, F>(arr, lo, hi, matches, scratch, log); }
    if 2 * (n / block) <= work.buffer.len() { return through_blocks(arr, lo, hi, block, matches, work, log); }
    let mid = lo + n / 2;
    let left = by_blocks::<M, R, T, U, F>(arr, lo, mid, matches, work, scratch, log);
    let right = by_blocks::<M, R, T, U, F>(arr, mid, hi, matches, work, scratch, log);
    rotate_region::<R, T, U>(arr, left, mid, right, scratch, log);
    left + (right - mid)
}

// The moves between the array and the buffer. A buffer that keeps its elements
// trades one for the array's, so that an emptied cell of the array holds a
// buffer element until it is filled again; one that does not is written to.
fn exchange<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], i: usize, buffer: &mut [T], j: usize, log: &mut U) {
    let (ours, theirs) = (arr[i], buffer[j]);
    log.write_data(arr, i, theirs);
    log.write_data(buffer, j, ours);
}

fn lend<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], i: usize, buffer: &mut [T], j: usize, keeps: bool, log: &mut U) {
    if keeps { exchange(arr, i, buffer, j, log) } else { log.write_accross(arr, i, buffer, j) }
}

fn take<T: Ord + Copy, U: ?Sized + SortLogger<T>>(buffer: &mut [T], j: usize, arr: &mut [T], i: usize, keeps: bool, log: &mut U) {
    if keeps { exchange(arr, i, buffer, j, log) } else { log.write_accross(buffer, j, arr, i) }
}

// The element at `from` to the emptied cell `to`, which is emptied in turn.
fn carry<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], to: usize, from: usize, keeps: bool, log: &mut U) {
    if keeps { log.swap(arr, to, from) } else { log.write(arr, to, from) }
}

// `arr[lo..hi]` holds no more blocks of `block` than half the marks. The low
// half of the buffer gathers the matches and the high half the rest.
fn through_blocks<M: Marks, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    block: usize,
    matches: &F,
    work: &mut Workspace<M, T>,
    log: &mut U,
) -> usize {
    let (buffer, marks, carved, keeps) = (&mut *work.buffer, &mut *work.marks, &mut *work.carved, work.keeps);
    // The scan has emptied as many cells behind it as the buffer holds, so a
    // full half always has a block of them to go to, at `write`. The mark of a
    // block of the rest is set.
    let (mut write, mut lows, mut highs, mut low_blocks) = (lo, 0, 0, 0);
    for read in lo..hi {
        let low = matches(log, arr, read);
        lend(arr, read, buffer, if low { lows } else { block + highs }, keeps, log);
        if low { lows += 1 } else { highs += 1 }
        if lows == block || highs == block {
            let half = if lows == block { 0 } else { block };
            if half == block { M::mark(marks, carved, (write - lo) / block, log); }
            for k in 0..block { take(buffer, half + k, arr, write + k, keeps, log); }
            write += block;
            if half == 0 { (lows, low_blocks) = (0, low_blocks + 1) } else { highs = 0 }
        }
    }
    // What is left of the rest goes last, which frees the high half to carry
    // a block.
    for k in 0..highs { take(buffer, block + k, arr, hi - highs + k, keeps, log); }

    // The block bound for the place `to` is the next of its kind in the order
    // they filled: the matches' blocks first, then the rest's. The kind marks
    // stay with the place a block filled at, wherever the block is moved.
    let blocks = (write - lo) / block;
    let source = |marks: &M::Store, carved: &[T], to: usize, log: &mut U| {
        let (high, rank) = if to < low_blocks { (false, to) } else { (true, to - low_blocks) };
        (0..blocks).filter(|&i| M::is_marked(marks, carved, i, log) == high).nth(rank).expect("a block for every place")
    };
    // A cycle is walked from its first place: that block waits in the buffer,
    // each place takes the block bound for it, and the last takes the one that
    // waited. A place is marked as it is filled, at `blocks + place`.
    for start in 0..blocks {
        if M::is_marked(marks, carved, blocks + start, log) { continue; }
        let (mut hole, mut from) = (start, source(marks, carved, start, log));
        if from == start { continue; }
        for k in 0..block { lend(arr, lo + start * block + k, buffer, block + k, keeps, log); }
        loop {
            M::mark(marks, carved, blocks + hole, log);
            if from == start { break; }
            for k in 0..block { carry(arr, lo + hole * block + k, lo + from * block + k, keeps, log); }
            (hole, from) = (from, source(marks, carved, from, log));
        }
        for k in 0..block { take(buffer, block + k, arr, lo + hole * block + k, keeps, log); }
    }
    for i in 0..2 * blocks {
        if M::is_marked(marks, carved, i, log) { M::unmark(marks, carved, i, log); }
    }

    // What is left of the matches goes between the two kinds of block: the
    // rest's blocks move up by as many cells, and the cells they leave fill.
    let split = lo + low_blocks * block;
    if lows > 0 {
        for i in (split..write).rev() { carry(arr, i + lows, i, keeps, log); }
        for k in 0..lows { take(buffer, k, arr, split + k, keeps, log); }
    }
    split + lows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{item, items, pairs, Item, Rng};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::{NoOpLogger, SortLog, SortLogger};

    // The expected outcome, spelled out: the elements the predicate picks in
    // order, then the rest in order; and for the three way split, below, equal
    // and above, each in order.
    fn reference(arr: &[Item], picked: impl Fn(u32) -> bool) -> Vec<(u32, u32)> {
        let first = arr.iter().filter(|x| picked(x.key));
        let rest = arr.iter().filter(|x| !picked(x.key));
        first.chain(rest).map(|x| (x.key, x.tag)).collect()
    }

    fn three_way(arr: &[Item], pivot: u32) -> Vec<(u32, u32)> {
        let below = arr.iter().filter(|x| x.key < pivot);
        let equal = arr.iter().filter(|x| x.key == pivot);
        let above = arr.iter().filter(|x| x.key > pivot);
        below.chain(equal).chain(above).map(|x| (x.key, x.tag)).collect()
    }

    // Counts comparisons, to pin one per element.
    #[derive(Default)]
    struct Cmps(usize);
    impl SortLogger<Item> for Cmps {
        fn log(&mut self, event: SortLog<Item>) {
            if matches!(event, SortLog::CmpData { .. } | SortLog::CmpInArr { .. }) { self.0 += 1 }
        }
    }

    // Every partition, under every rotation, with the scratch the rotation asks
    // for or the `cells` given, leaves the elements before `lo` alone and the
    // region as the reference says.
    fn matches_the_reference<P: StablePartition, R: Rotation>(cells: Option<usize>) {
        let mut rng = Rng(0xB1A5);
        for round in 0..400 {
            let n = 1 + rng.next(60);
            let values = 1 + rng.next(n + 3);
            let arr0 = items(&(0..n).map(|_| rng.next(values) as u32).collect::<Vec<_>>());
            let lo = rng.next(3).min(n - 1);
            let pivot = rng.next(values + 2) as u32;
            let mut arr = arr0.clone();
            let mut scratch = vec![item(0); cells.unwrap_or(R::scratch_size(n))];
            let below = arr0[lo..].iter().filter(|x| x.key < pivot).count();
            let not_above = arr0[lo..].iter().filter(|x| x.key <= pivot).count();
            let name = format!("{}, {}: round {round}: n={n} values={values} pivot={pivot} lo={lo}", P::NAME, R::NAME);
            let mut log = Cmps::default();
            let split = P::partition::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut log);
            assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{name}: touched the prefix");
            assert_eq!(pairs(&arr[lo..]), reference(&arr0[lo..], |k| k < pivot), "{name}: below");
            assert_eq!(split, lo + below, "{name}: the split is off");
            assert_eq!(log.0, n - lo, "{name}: more than one comparison per element");
            let mut arr = arr0.clone();
            let split = P::partition_at_most::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut NoOpLogger);
            assert_eq!(pairs(&arr[lo..]), reference(&arr0[lo..], |k| k <= pivot), "{name}: not above");
            assert_eq!(split, lo + not_above, "{name}: the at most split is off");
            let mut arr = arr0.clone();
            let (equal, above) = P::partition_three_way::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut NoOpLogger);
            assert_eq!(pairs(&arr[lo..]), three_way(&arr0[lo..], pivot), "{name}: three way");
            assert_eq!((equal, above), (lo + below, lo + not_above), "{name}: the three way splits are off");
        }
    }

    #[test]
    fn the_rotation_partition_matches_the_reference() {
        matches_the_reference::<RotationPartition, ReversalRotation>(None);
        matches_the_reference::<RotationPartition, TrinityRotation>(None);
        matches_the_reference::<RotationPartition, AuxiliaryRotation>(None);
    }

    // A scratch of any size: none, a few cells so the leaves are buffered and the
    // levels above them rotate, and the whole region in one pass.
    #[test]
    fn the_rotation_partition_takes_any_scratch() {
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(0));
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(5));
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(64));
    }

    #[test]
    fn the_sliding_partition_matches_the_reference() {
        matches_the_reference::<SlidingPartition, ReversalRotation>(None);
        matches_the_reference::<SlidingPartition, TrinityRotation>(None);
        matches_the_reference::<SlidingPartition, AuxiliaryRotation>(None);
    }

    #[test]
    fn an_empty_region_and_an_unmoved_one_cost_nothing() {
        let mut arr = items(&[3, 1, 2]);
        let empty = RotationPartition::partition::<ReversalRotation, _, _>(&mut arr, 2, 2, item(9), &mut [], &mut NoOpLogger);
        assert_eq!(empty, 2);
        let all = SlidingPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, 3, item(9), &mut [], &mut NoOpLogger);
        assert_eq!((all, pairs(&arr)), (3, vec![(3, 0), (1, 1), (2, 2)]));
        let none = SlidingPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, 3, item(0), &mut [], &mut NoOpLogger);
        assert_eq!((none, pairs(&arr)), (0, vec![(3, 0), (1, 1), (2, 2)]));
    }
    // Buffer cells carry tags from 10000 up and the cells marks are carved from
    // tags from 20000 up, the latter distinct and sorted.
    fn tagged(cells: usize, from: u32, distinct: bool) -> Vec<Item> {
        (0..cells).map(|i| Item { key: if distinct { i as u32 } else { 3 }, tag: from + i as u32 }).collect()
    }

    // The block partition through a workspace of `cells`, kept or not, under
    // the marks `M`: the region as the reference says, the buffer with every
    // element it had, and the marks clear and their cells as they were.
    fn blocks_match_the_reference<M: Marks>(keeps: bool) {
        let mut rng = Rng(0xB10C);
        for round in 0..300 {
            let n = 1 + rng.next(if round % 3 == 0 { 900 } else { 120 });
            let cells = [0, 5, 16, 17, 24, 40, 64][rng.next(7)];
            let values = 1 + rng.next(n + 3);
            let arr0 = items(&(0..n).map(|_| rng.next(values) as u32).collect::<Vec<_>>());
            let lo = rng.next(3).min(n - 1);
            let pivot = rng.next(values + 2) as u32;
            let name = format!("{}, keeps {keeps}: round {round}: n={n} cells={cells} values={values} pivot={pivot} lo={lo}", M::NAME);
            let run = |split: &dyn Fn(&mut [Item], &mut Workspace<M, Item>) -> Vec<usize>| {
                let mut arr = arr0.clone();
                let (mut buffer, mut carved) = (tagged(cells, 10_000, false), tagged(cells * M::CARVED, 20_000, true));
                let mut marks = M::create::<Item, _>(cells, &mut NoOpLogger);
                let splits = split(&mut arr, &mut Workspace { buffer: &mut buffer, keeps, marks: &mut marks, carved: &mut carved });
                assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{name}: touched the prefix");
                assert!((0..cells).all(|i| !M::is_marked(&marks, &carved, i, &mut NoOpLogger)), "{name}: a mark is left set");
                assert_eq!(pairs(&carved), pairs(&tagged(cells * M::CARVED, 20_000, true)), "{name}: the carved marks moved");
                if keeps {
                    let mut tags: Vec<u32> = buffer.iter().map(|x| x.tag).collect();
                    tags.sort();
                    assert_eq!(tags, (0..cells as u32).map(|i| 10_000 + i).collect::<Vec<_>>(), "{name}: the buffer lost an element");
                }
                (arr, splits)
            };
            let below = arr0[lo..].iter().filter(|x| x.key < pivot).count();
            let not_above = arr0[lo..].iter().filter(|x| x.key <= pivot).count();
            let (arr, splits) = run(&|arr, work| vec![BlockPartition::partition_lent::<M, ReversalRotation, _, _>(arr, lo, n, item(pivot), work, &mut [], &mut NoOpLogger)]);
            assert_eq!((pairs(&arr[lo..]), splits), (reference(&arr0[lo..], |k| k < pivot), vec![lo + below]), "{name}: below");
            let (arr, splits) = run(&|arr, work| vec![BlockPartition::partition_at_most_lent::<M, ReversalRotation, _, _>(arr, lo, n, item(pivot), work, &mut [], &mut NoOpLogger)]);
            assert_eq!((pairs(&arr[lo..]), splits), (reference(&arr0[lo..], |k| k <= pivot), vec![lo + not_above]), "{name}: not above");
            let (arr, splits) = run(&|arr, work| {
                let (equal, above) = BlockPartition::partition_three_way_lent::<M, ReversalRotation, _, _>(arr, lo, n, item(pivot), work, &mut [], &mut NoOpLogger);
                vec![equal, above]
            });
            assert_eq!((pairs(&arr[lo..]), splits), (three_way(&arr0[lo..], pivot), vec![lo + below, lo + not_above]), "{name}: three way");
        }
    }

    #[test]
    fn the_block_partition_matches_the_reference_through_any_workspace() {
        for keeps in [true, false] {
            blocks_match_the_reference::<scratchspace_lib::ByteMarks>(keeps);
            blocks_match_the_reference::<scratchspace_lib::BitMarks>(keeps);
            blocks_match_the_reference::<scratchspace_lib::InArrayMarks>(keeps);
        }
    }

    #[test]
    fn the_block_partition_without_a_workspace_is_the_rotation_partition() {
        matches_the_reference::<BlockPartition, ReversalRotation>(None);
        matches_the_reference::<BlockPartition, AuxiliaryRotation>(None);
    }

    // Counts element writes, a swap as two.
    #[derive(Default)]
    struct Writes(usize);
    impl SortLogger<Item> for Writes {
        fn log(&mut self, event: SortLog<Item>) {
            match event {
                SortLog::Swap { .. } => self.0 += 2,
                SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => self.0 += 1,
                _ => {}
            }
        }
    }

    // Through a workspace of twice the square root the writes per element do
    // not grow with the region, where the rotation partition's grow by its log.
    #[test]
    fn the_block_partition_is_linear_through_a_square_root() {
        let per_element = |n: usize, blocks: bool| {
            let mut rng = Rng(0x11EA + n as u64);
            let mut arr = items(&(0..n).map(|_| rng.next(1000) as u32).collect::<Vec<_>>());
            let cells = 2 * n.isqrt();
            let (mut buffer, mut marks, mut log) = (tagged(cells, 10_000, false), vec![0u8; cells], Writes::default());
            let mut work = Workspace::<scratchspace_lib::ByteMarks, Item> { buffer: &mut buffer, keeps: true, marks: &mut marks, carved: &mut [] };
            if blocks {
                BlockPartition::partition_lent::<_, ReversalRotation, _, _>(&mut arr, 0, n, item(500), &mut work, &mut [], &mut log);
            } else {
                RotationPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, item(500), &mut [], &mut log);
            }
            log.0 as f64 / n as f64
        };
        let (small, large) = (per_element(1 << 10, true), per_element(1 << 16, true));
        assert!(large < small * 1.15 && large < 12.0, "blocks: {small} then {large}");
        assert!(per_element(1 << 16, false) > per_element(1 << 10, false) * 1.4, "the rotation partition did not grow");
    }
}
