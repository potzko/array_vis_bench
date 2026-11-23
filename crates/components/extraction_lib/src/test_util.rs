//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator, the
//! pair view of an array, the expected outcome of a partition, a tally of the
//! logged moves and comparisons, `Needy`, a partition that insists on its
//! scratch, and the battery the block partitions share: the shapes of input
//! (`types`), the comparison with the reference over every size
//! (`matches_the_reference`), the cost of one pass (`cost`) and the tests of
//! that cost, which a partition's own test module calls with itself.

use core::cmp::Ordering;

use crate::blocks::ceil_log2;
use crate::RotationPartition;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use rotation_reversal::ReversalRotation;
use sort_logger::{split_sections, NoOpLogger, SortLog, SortLogger};

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub key: u32,
    pub tag: u32,
}
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool { self.key == other.key }
}
impl Eq for Item {}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) }
}

pub fn items(keys: &[u32]) -> Vec<Item> {
    keys.iter().zip(0u32..).map(|(&key, tag)| Item { key, tag }).collect()
}

pub fn item(key: u32) -> Item { Item { key, tag: 0 } }

pub fn pairs(arr: &[Item]) -> Vec<(u32, u32)> { arr.iter().map(|x| (x.key, x.tag)).collect() }

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self, m: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % m
    }
}

/// Partitions by rotation, wants a scratch of its own as long as the region and
/// fails when it is handed a shorter one.
pub struct Needy;
impl StablePartition for Needy {
    const NAME: &'static str = "needy";
    const MOVES: Complexity = Complexity::N_LOG_N;

    fn scratch_size(n: usize) -> usize { n }

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        assert!(scratch.len() >= hi - lo, "a scratch of {} for a region of {}", scratch.len(), hi - lo);
        RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        assert!(scratch.len() >= hi - lo, "a scratch of {} for a region of {}", scratch.len(), hi - lo);
        RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
    }
}

/// Partitions by rotation and keeps words of its own, three sections with an
/// empty one between: it checks their length on every call and writes the size of
/// the region into the last word of the last section, through the logger.
pub struct Wordy;
impl StablePartition for Wordy {
    const NAME: &'static str = "wordy";
    const MOVES: Complexity = Complexity::N_LOG_N;
    const SCRATCH_SECTIONS: &'static [usize] = &[3, 0, 5];

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        assert_eq!(words.len(), 8, "the words of a call");
        let [_, _, tail] = split_sections(&mut *words, [3, 0, 5]);
        log.write_data_u(tail, 4, hi - lo);
        RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        assert_eq!(words.len(), 8, "the words of a call");
        let [_, _, tail] = split_sections(&mut *words, [3, 0, 5]);
        log.write_data_u(tail, 4, hi - lo);
        RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
    }
}

/// The words a partition asks for, blank, for a test that does not register them.
pub fn words_of<P: StablePartition>() -> Vec<usize> { vec![0; P::SCRATCH_SECTIONS.iter().sum()] }

/// The expected outcome of a partition, spelled out: the elements the predicate
/// picks in order, then the rest in order.
pub fn reference(arr: &[Item], picked: impl Fn(u32) -> bool) -> Vec<(u32, u32)> {
    let first = arr.iter().filter(|x| picked(x.key));
    let rest = arr.iter().filter(|x| !picked(x.key));
    first.chain(rest).map(|x| (x.key, x.tag)).collect()
}

/// The expected three way split: below, equal and above, each in order.
pub fn three_way(arr: &[Item], pivot: u32) -> Vec<(u32, u32)> {
    let below = arr.iter().filter(|x| x.key < pivot);
    let equal = arr.iter().filter(|x| x.key == pivot);
    let above = arr.iter().filter(|x| x.key > pivot);
    below.chain(equal).chain(above).map(|x| (x.key, x.tag)).collect()
}

/// Counts the logged moves and comparisons, to read a growth rate off a run.
#[derive(Default)]
pub struct Tally {
    pub moves: usize,
    pub cmps: usize,
}
impl<T: Copy + Ord> SortLogger<T> for Tally {
    fn log(&mut self, event: SortLog<T>) {
        match event {
            SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } | SortLog::WriteDataU { .. } => {
                self.moves += 1
            }
            SortLog::CmpInArr { .. } | SortLog::CmpData { .. } | SortLog::CmpDataU { .. } | SortLog::CmpAcrossArrs { .. } => {
                self.cmps += 1
            }
            _ => {}
        }
    }
}

/// Replays the events onto copies of the arrays, so that comparing the copy of
/// the array that was partitioned with the array itself shows that no move went
/// unlogged. An event finds its array by address, as the visualiser does, so a
/// sub slice of the array, which is what a rotation is handed, resolves to the
/// right copy and offset.
pub struct Replay {
    arrays: Vec<(usize, Vec<usize>)>,
}
impl Replay {
    pub fn of(arr: &[usize]) -> Self { Replay { arrays: vec![(arr.as_ptr() as usize, arr.to_vec())] } }

    pub fn main(&self) -> &[usize] { &self.arrays[0].1 }

    // The copy an address falls in, and the offset of the address in it.
    fn at(&mut self, name: usize) -> (&mut Vec<usize>, usize) {
        let size = std::mem::size_of::<usize>();
        let (base, copy) = self
            .arrays
            .iter_mut()
            .filter(|(base, copy)| *base <= name && name < *base + copy.len() * size)
            .last()
            .expect("an event names an array that is not live");
        (copy, (name - *base) / size)
    }
}
impl SortLogger<usize> for Replay {
    fn log(&mut self, event: SortLog<usize>) {
        match event {
            SortLog::Swap { name, ind_a, ind_b } => {
                let (copy, off) = self.at(name);
                copy.swap(ind_a + off, ind_b + off);
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => {
                let (copy, off) = self.at(name);
                copy[ind_a + off] = copy[ind_b + off];
            }
            SortLog::WriteData { name, ind, data } | SortLog::WriteDataU { name, ind, data } => {
                let (copy, off) = self.at(name);
                copy[ind + off] = data;
            }
            SortLog::CreateAuxArrT { name, length } | SortLog::CreateAuxArr { name, length } => {
                self.arrays.push((name, vec![0; length]));
            }
            SortLog::FreeAuxArr { name } => self.arrays.retain(|(base, _)| *base != name),
            _ => {}
        }
    }
}

pub const PIVOT: u32 = 5;
pub const SHAPES: usize = 15;

// A key of the front type, below the pivot, or of the back type, at or
// above it, so that the three splits all have something to tell apart.
pub fn key(front: bool, rng: &mut Rng) -> u32 {
    if front { rng.next(PIVOT as usize) as u32 } else { PIVOT + rng.next(4) as u32 }
}

// Which elements of a region of `n` are of the front type, in each shape of
// input that blocks of `b` cells can take: one type throughout, either type in the
// minority, whole blocks alternating or in random order, runs, sorted and
// reversed, and a lone block or element of the other type.
pub fn types(shape: usize, n: usize, b: usize, rng: &mut Rng) -> Vec<bool> {
    let b = b.max(1);
    let at = rng.next(n.max(1));
    let block_kinds: Vec<bool> = (0..n.div_ceil(b)).map(|_| rng.next(2) == 0).collect();
    let runs = {
        let mut kinds = Vec::new();
        while kinds.len() < n {
            let kind = rng.next(2) == 0;
            (0..1 + rng.next(3 * b)).for_each(|_| kinds.push(kind));
        }
        kinds.truncate(n);
        kinds
    };
    match shape {
        0 => vec![true; n],
        1 => vec![false; n],
        2 => (0..n).map(|_| rng.next(2) == 0).collect(),
        3 => (0..n).map(|_| rng.next(16) == 0).collect(),
        4 => (0..n).map(|_| rng.next(16) != 0).collect(),
        5 => (0..n).map(|i| (i / b) % 2 == 0).collect(),
        6 => (0..n).map(|i| i % 2 == 0).collect(),
        7 => (0..n).map(|i| i < n / 2).collect(),
        8 => (0..n).map(|i| i >= n / 2).collect(),
        9 => (0..n).map(|i| !(at..at + b).contains(&i)).collect(),
        10 => (0..n).map(|i| (at..at + b).contains(&i)).collect(),
        11 => (0..n).map(|i| i != at).collect(),
        12 => (0..n).map(|i| i == at).collect(),
        13 => (0..n).map(|i| block_kinds[i / b]).collect(),
        _ => runs,
    }
}

pub fn expect(arr: &[Item], arr0: &[Item], lo: usize, hi: usize, region: Vec<(u32, u32)>, name: &str) {
    assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{name}: touched the prefix");
    assert_eq!(pairs(&arr[lo..hi]), region, "{name}: the region");
    assert_eq!(pairs(&arr[hi..]), pairs(&arr0[hi..]), "{name}: touched the suffix");
}

// Every call, of the three splits, matches the reference, on every shape and
// every region size to 300 and a few above, with guard cells either side and
// the scratch the rotation or the partition asks for, or `extra` cells more,
// the shapes laid out in blocks of `block(n)` cells.
pub fn matches_the_reference<P: StablePartition, R: Rotation>(extra: usize, block: fn(usize) -> usize) {
    let mut rng = Rng(0xE0C0DED);
    for n in (0..=300).chain([511, 512, 513, 1000, 1024, 1025, 4093, 4096]) {
        for shape in 0..SHAPES {
            let (lo, front) = (rng.next(3), types(shape, n, block(n), &mut rng));
            let guard: Vec<u32> = (0..lo).map(|_| rng.next(9) as u32).collect();
            let keys: Vec<u32> = guard.into_iter().chain(front.iter().map(|&f| key(f, &mut rng))).chain([1, 8]).collect();
            let arr0 = items(&keys);
            let hi = lo + n;
            let mut scratch = vec![item(u32::MAX); R::scratch_size(n).max(P::scratch_size(n)) + extra];
            let mut words = words_of::<P>();
            let name = format!("{}, {}: n={n} shape={shape} lo={lo} extra={extra}", P::NAME, R::NAME);
            let below = arr0[lo..hi].iter().filter(|x| x.key < PIVOT).count();
            let not_above = arr0[lo..hi].iter().filter(|x| x.key <= PIVOT).count();
            let mut arr = arr0.clone();
            let split = P::partition::<R, _, _>(&mut arr, lo, hi, item(PIVOT), &mut scratch, &mut words, &mut NoOpLogger);
            expect(&arr, &arr0, lo, hi, reference(&arr0[lo..hi], |k| k < PIVOT), &name);
            assert_eq!(split, lo + below, "{name}: the split is off");
            let mut arr = arr0.clone();
            let split = P::partition_at_most::<R, _, _>(&mut arr, lo, hi, item(PIVOT), &mut scratch, &mut words, &mut NoOpLogger);
            expect(&arr, &arr0, lo, hi, reference(&arr0[lo..hi], |k| k <= PIVOT), &name);
            assert_eq!(split, lo + not_above, "{name}: the at most split is off");
            let mut arr = arr0.clone();
            let ends = P::partition_three_way::<R, _, _>(&mut arr, lo, hi, item(PIVOT), &mut scratch, &mut words, &mut NoOpLogger);
            expect(&arr, &arr0, lo, hi, three_way(&arr0[lo..hi], PIVOT), &name);
            assert_eq!(ends, (lo + below, lo + not_above), "{name}: the three way splits are off");
        }
    }
}

// Arrays of only two, three and four distinct keys, in a few skews, around every
// pivot from below the least key to above the greatest: a partition that writes
// its block ranks into the elements must need no element to be distinct from
// another, since the pivot tells the two elements it swaps apart.
pub fn few_distinct_values_match_the_reference<P: StablePartition, R: Rotation>() {
    let mut rng = Rng(0xF3D);
    for n in (0..=200).chain([255, 256, 257, 511, 1000, 1024, 3000, 4096]) {
        for values in 2..=4u32 {
            for skew in 0..4 {
                let mut run = (0, 0);
                let keys: Vec<u32> = (0..n)
                    .map(|_| match skew {
                        0 => rng.next(values as usize) as u32,
                        1 => if rng.next(8) == 0 { rng.next(values as usize) as u32 } else { 0 },
                        2 => if rng.next(8) == 0 { rng.next(values as usize) as u32 } else { values - 1 },
                        _ => {
                            if run.0 == 0 { run = (1 + rng.next(40), rng.next(values as usize) as u32) }
                            run.0 -= 1;
                            run.1
                        }
                    })
                    .collect();
                let arr0 = items(&keys);
                let mut scratch = vec![item(u32::MAX); R::scratch_size(n).max(P::scratch_size(n))];
                let mut words = words_of::<P>();
                for pivot in 0..=values {
                    let name = format!("{}, {}: n={n} values={values} skew={skew} pivot={pivot}", P::NAME, R::NAME);
                    let below = arr0.iter().filter(|x| x.key < pivot).count();
                    let not_above = arr0.iter().filter(|x| x.key <= pivot).count();
                    let mut arr = arr0.clone();
                    let split = P::partition::<R, _, _>(&mut arr, 0, n, item(pivot), &mut scratch, &mut words, &mut NoOpLogger);
                    assert_eq!((pairs(&arr), split), (reference(&arr0, |k| k < pivot), below), "{name}: below");
                    let mut arr = arr0.clone();
                    let split = P::partition_at_most::<R, _, _>(&mut arr, 0, n, item(pivot), &mut scratch, &mut words, &mut NoOpLogger);
                    assert_eq!((pairs(&arr), split), (reference(&arr0, |k| k <= pivot), not_above), "{name}: at most");
                    let mut arr = arr0.clone();
                    let ends = P::partition_three_way::<R, _, _>(&mut arr, 0, n, item(pivot), &mut scratch, &mut words, &mut NoOpLogger);
                    assert_eq!((pairs(&arr), ends), (three_way(&arr0, pivot), (below, not_above)), "{name}: three way");
                }
            }
        }
    }
}

// Logged moves and comparisons per element of one partition of random
// `usize` keys of a shape's `types` in blocks of `b` cells.
pub fn cost<P: StablePartition, R: Rotation>(n: usize, shape: usize, b: usize) -> (f64, f64) {
    let mut rng = Rng(7 + n as u64);
    let mut arr: Vec<usize> = types(shape, n, b, &mut rng).iter().map(|&f| if f { rng.next(n) } else { n + rng.next(n) }).collect();
    let mut scratch = vec![0usize; R::scratch_size(n).max(P::scratch_size(n))];
    let mut words = words_of::<P>();
    let mut tally = Tally::default();
    let split = P::partition::<R, _, _>(&mut arr, 0, n, n, &mut scratch, &mut words, &mut tally);
    assert!(arr[..split].iter().all(|&x| x < n) && arr[split..].iter().all(|&x| x >= n));
    (tally.moves as f64 / n as f64, tally.cmps as f64 / n as f64)
}

// The block length leaves the cell that tells a block's type alone: the code
// cells are `ceil(log2(minority blocks))` of the cells, the minority is at most
// half of the blocks, and a block is longer than that.
pub fn code_cells_fit(block: fn(usize) -> usize) {
    for n in (2..20_000).chain([1 << 20, (1 << 20) + 1, 1 << 30, usize::MAX >> 1]) {
        let b = block(n);
        let minority = n / b / 2;
        assert!(b >= 2 && (minority < 2 || ceil_log2(minority) < b), "n={n}: {b} cells for {minority} minority blocks");
    }
}

// Moves and comparisons that count are the logged ones, and a cell of the
// scratch and a word are written before they are read: both are full of garbage
// the replay's copy of them lacks, so a read of an unwritten cell would leave the
// two arrays apart.
pub fn replays_without_a_gap<P: StablePartition>(block: fn(usize) -> usize) {
    const BACK: usize = 1 << 20;
    let mut rng = Rng(0x10661);
    for n in [3usize, 10, 33, 100, 257, 1000, 3001] {
        for shape in 0..SHAPES {
            let mut arr: Vec<usize> = types(shape, n, block(n), &mut rng).iter().zip(0..).map(|(&f, i)| if f { i } else { BACK + i }).collect();
            let mut replay = Replay::of(&arr);
            let mut scratch = vec![usize::MAX; ReversalRotation::scratch_size(n).max(P::scratch_size(n))];
            replay.log_aux_arr_t(&scratch);
            let mut words = replay.create_aux_arr_split(P::SCRATCH_SECTIONS);
            words.fill(usize::MAX);
            let split = P::partition::<ReversalRotation, _, _>(&mut arr, 0, n, BACK, &mut scratch, &mut words, &mut replay);
            assert!(arr[..split].iter().all(|&x| x < BACK) && arr[split..].iter().all(|&x| x >= BACK), "n={n} shape={shape}");
            assert!(arr[..split].windows(2).all(|w| w[0] < w[1]) && arr[split..].windows(2).all(|w| w[0] < w[1]), "n={n} shape={shape}: order");
            assert_eq!(replay.main(), &arr[..], "n={n} shape={shape}: a move went unlogged or an unwritten cell was read");
        }
    }
}

// The moves per element do not grow with the size from `small` elements to
// 2^16, which a partition by rotation's `log n` would, and stay under a few and
// below the rotation partition's.
pub fn moves_are_flat_and_below_the_rotation_partition<P: StablePartition>(block: fn(usize) -> usize, small: usize) {
    let (small_moves, _) = cost::<P, ReversalRotation>(small, 2, block(small));
    let (large, cmps) = cost::<P, ReversalRotation>(1 << 16, 2, block(1 << 16));
    let (rotation, _) = cost::<RotationPartition, ReversalRotation>(1 << 16, 2, block(1 << 16));
    assert!(large < 1.3 * small_moves, "{small_moves} moves per element at {small} and {large} at 2^16");
    assert!(large < 5.0, "{large} moves per element at 2^16");
    assert!(cmps < 1.3, "{cmps} comparisons per element at 2^16");
    assert!(2.0 * large < rotation, "{large} moves per element against the rotation partition's {rotation}");
}

pub fn every_shape_is_cheap<P: StablePartition>(block: fn(usize) -> usize) {
    for shape in 0..SHAPES {
        let (moves, cmps) = cost::<P, ReversalRotation>(1 << 14, shape, block(1 << 14));
        assert!(moves < 5.0 && cmps < 1.5, "shape {shape}: {moves} moves and {cmps} comparisons per element");
    }
}

// A region already partitioned, or with nothing to move to the front, costs its
// comparisons and no move: every element is read once and none written.
pub fn nothing_to_move_costs_comparisons_only<P: StablePartition>(block: fn(usize) -> usize) {
    for n in [100usize, 1 << 10, 1 << 14, 40_000] {
        for (shape, name) in [(0, "all front"), (1, "all back"), (7, "sorted")] {
            let mut rng = Rng(5);
            let types = types(shape, n, block(n), &mut rng);
            let mut arr: Vec<usize> = types.iter().enumerate().map(|(i, &f)| if f { i } else { n + i }).collect();
            let want = arr.clone();
            let mut scratch = vec![0usize; ReversalRotation::scratch_size(n).max(P::scratch_size(n))];
            let mut words = words_of::<P>();
            let mut tally = Tally::default();
            let split = P::partition::<ReversalRotation, _, _>(&mut arr, 0, n, n, &mut scratch, &mut words, &mut tally);
            assert_eq!(arr, want, "{name}, n={n}: moved");
            assert_eq!(split, types.iter().filter(|&&f| f).count(), "{name}, n={n}: the split");
            assert_eq!((tally.moves, tally.cmps <= n + 2), (0, true), "{name}, n={n}: {} moves, {} comparisons", tally.moves, tally.cmps);
        }
    }
}

// The ends are trimmed, and what is between them partitioned: an eighth in
// front, a quarter shuffled and the rest in back cost the quarter alone.
pub fn ends_in_place_cost_no_moves<P: StablePartition>() {
    let n = 1 << 14;
    let mut rng = Rng(11);
    let middle: Vec<usize> = (0..n / 4).map(|i| if rng.next(2) == 0 { i } else { n + i }).collect();
    let mut arr: Vec<usize> = (0..n / 8).chain(middle).chain((0..5 * n / 8).map(|i| n + i)).collect();
    let mut scratch = vec![0usize; ReversalRotation::scratch_size(n).max(P::scratch_size(n))];
    let mut words = words_of::<P>();
    let mut tally = Tally::default();
    let split = P::partition::<ReversalRotation, _, _>(&mut arr, 0, n, n, &mut scratch, &mut words, &mut tally);
    assert!(arr[..split].iter().all(|&x| x < n) && arr[split..].iter().all(|&x| x >= n));
    assert!(tally.moves < n, "{} moves for a middle of {}", tally.moves, n / 4);
    assert!(tally.cmps < n + n / 16, "{} comparisons", tally.cmps);
}

pub fn empty_and_lone_regions_cost_nothing<P: StablePartition>() {
    let mut arr = items(&[3, 1]);
    let mut tally = Tally::default();
    let mut words = words_of::<P>();
    let empty = P::partition::<ReversalRotation, _, _>(&mut arr, 1, 1, item(2), &mut [], &mut words, &mut tally);
    assert_eq!((empty, tally.moves, tally.cmps), (1, 0, 0));
    let lone = P::partition::<ReversalRotation, _, _>(&mut arr, 1, 2, item(2), &mut [], &mut words, &mut tally);
    assert_eq!((lone, tally.moves, tally.cmps), (2, 0, 1));
}
