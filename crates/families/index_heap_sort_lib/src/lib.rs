//! Index heap sort, `IndexHeapSort`: a stable heap sort that pays one word per
//! element. The heap holds indices into the array, ordered by the element at
//! each index and then by the index, so equal elements are distinct to the heap
//! and come out in their original order. The pops leave the stable sorting
//! permutation in the heap, and `permute` applies it to the array by following
//! its cycles, so an element is written once at most.

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct IndexHeapSort;

impl IndexHeapSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let mut heap = log.create_aux_arr(n);
        for i in 0..n { log.write_data_u(&mut heap, i, i); }
        for parent in (0..n / 2).rev() { sift_down(arr, &mut heap, parent, n, log); }
        for last in (1..n).rev() {
            swap_slots(&mut heap, 0, last, log);
            sift_down(arr, &mut heap, 0, last, log);
        }
        permute(arr, &mut heap, log);
        log.free_aux_arr(&heap);
    }
}

array_vis_bench_traits::direct_sort!(IndexHeapSort);

// The pops cost `n log n` comparisons on every input, whatever its order.
impl HasTimeBounds for IndexHeapSort {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl HasSpace for IndexHeapSort {
    const SPACE: Complexity = Complexity::N1;
}

// The index is the secondary key, so no two elements are equal to the heap and
// it pops them by key and then by index, which is their original order.
impl HasStability for IndexHeapSort {
    const STABLE: bool = true;
}

// Whether slot `a` of the heap holds a position that sorts before the one slot
// `b` holds: by element, then by position. The position is the tie break that
// makes equal elements distinct, plain bookkeeping and not a key comparison, so
// it is not logged.
fn slot_lt<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], heap: &[usize], a: usize, b: usize, log: &mut U) -> bool {
    let (x, y) = (heap[a], heap[b]);
    log.cmp_lt(arr, x, y) || (!log.cmp_lt(arr, y, x) && x < y)
}

fn swap_slots<T: Ord + Copy, U: ?Sized + SortLogger<T>>(heap: &mut [usize], a: usize, b: usize, log: &mut U) {
    let (x, y) = (heap[a], heap[b]);
    log.write_data_u(heap, a, y);
    log.write_data_u(heap, b, x);
}

// Sift the slot `root` of the max heap `heap[..len]` down by swapping it with
// its larger child.
fn sift_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], heap: &mut [usize], mut root: usize, len: usize, log: &mut U) {
    while 2 * root + 1 < len {
        let mut child = 2 * root + 1;
        if child + 1 < len && slot_lt(arr, heap, child, child + 1, log) { child += 1; }
        if !slot_lt(arr, heap, root, child, log) { return; }
        swap_slots(heap, root, child, log);
        root = child;
    }
}

// `heap[k]` is the position of the element that belongs at `k`. Each cycle of
// that permutation is walked once: the element at its first slot is held, every
// other slot takes the element it points at, and the held one closes the cycle.
// A slot is marked done, by pointing at itself, after its successor is read.
fn permute<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], heap: &mut [usize], log: &mut U) {
    for start in 0..arr.len() {
        if heap[start] == start { continue; }
        let held = arr[start];
        let mut slot = start;
        while heap[slot] != start {
            let next = heap[slot];
            log.write(arr, slot, next);
            log.write_data_u(heap, slot, slot);
            slot = next;
        }
        log.write_data(arr, slot, held);
        log.write_data_u(heap, slot, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;
    use sort_logger::{NoOpLogger, SortLog};

    // Ordered by `key` alone, so `tag` records where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Item {
        key: u32,
        tag: u32,
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

    struct Rng(u64);
    impl Rng {
        fn below(&mut self, m: usize) -> usize {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 33) as usize % m
        }
    }

    fn pairs(items: &[Item]) -> Vec<(u32, u32)> { items.iter().map(|x| (x.key, x.tag)).collect() }

    // The shapes of input a sort sees. The random shapes of few values repeat
    // each key often, so equal elements are the common case.
    fn keys(kind: usize, n: usize, rng: &mut Rng) -> Vec<u32> {
        match kind {
            0 => (0..n).map(|_| rng.below(2) as u32).collect(),
            1 => (0..n).map(|_| rng.below(3) as u32).collect(),
            2 => (0..n).map(|_| rng.below(10) as u32).collect(),
            3 => (0..n).map(|_| rng.below(40) as u32).collect(),
            4 => (0..n as u32).collect(),
            5 => (0..n as u32).rev().collect(),
            6 => vec![5; n],
            _ => (0..n).map(|_| rng.below(n * 4) as u32).collect(),
        }
    }
    const KINDS: usize = 8;

    #[test]
    fn equal_elements_come_out_in_their_original_order() {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 31, 32, 33, 64, 100, 257, 600, 1500] {
            for kind in 0..KINDS {
                let items: Vec<Item> =
                    keys(kind, n, &mut rng).into_iter().zip(0..).map(|(key, tag)| Item { key, tag }).collect();
                let mut got = items.clone();
                IndexHeapSort::sort(&mut got, &mut NoOpLogger);
                let mut want = items;
                want.sort_by_key(|x| x.key);
                assert_eq!(pairs(&got), pairs(&want), "kind {kind}, n={n}");
            }
        }
    }

    // Replays the events onto copies of the arrays, so that comparing the copy of
    // the array that was sorted with the array itself shows that no move went
    // unlogged. An event finds its array by address, as the visualiser does, so
    // the word store resolves to its own copy.
    struct Replay {
        arrays: Vec<(usize, Vec<usize>)>,
    }
    impl Replay {
        fn of(arr: &[usize]) -> Self { Replay { arrays: vec![(arr.as_ptr() as usize, arr.to_vec())] } }

        fn main(&self) -> &[usize] { &self.arrays[0].1 }

        fn live(&self) -> usize { self.arrays.len() }

        fn at(&mut self, name: usize) -> &mut Vec<usize> {
            let size = std::mem::size_of::<usize>();
            let (_, copy) = self
                .arrays
                .iter_mut()
                .filter(|(base, copy)| *base <= name && name < *base + copy.len() * size)
                .last()
                .expect("an event names an array that is not live");
            copy
        }
    }
    impl SortLogger<usize> for Replay {
        fn log(&mut self, event: SortLog<usize>) {
            match event {
                SortLog::Swap { name, ind_a, ind_b } => self.at(name).swap(ind_a, ind_b),
                SortLog::WriteInArr { name, ind_a, ind_b } => {
                    let copy = self.at(name);
                    copy[ind_a] = copy[ind_b];
                }
                SortLog::WriteData { name, ind, data } | SortLog::WriteDataU { name, ind, data } => {
                    self.at(name)[ind] = data
                }
                SortLog::CreateAuxArrT { name, length } | SortLog::CreateAuxArr { name, length } => {
                    self.arrays.push((name, vec![0; length]));
                }
                SortLog::FreeAuxArr { name } => self.arrays.retain(|(base, _)| *base != name),
                _ => {}
            }
        }
    }

    #[test]
    fn every_move_is_logged() {
        let mut rng = Rng(0x10661);
        let mut arr: Vec<usize> = (0..300).map(|_| rng.below(40)).collect();
        let mut replay = Replay::of(&arr);
        IndexHeapSort::sort(&mut arr, &mut replay);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted");
        assert_eq!(replay.main(), &arr[..], "a move went unlogged");
        assert_eq!(replay.live(), 1, "the word store was not freed");
    }

    // Counts the comparisons, the writes of elements and the writes of words.
    #[derive(Default)]
    struct Tally {
        cmps: usize,
        element_writes: usize,
        word_writes: usize,
    }
    impl<T: Copy + Ord> SortLogger<T> for Tally {
        fn log(&mut self, event: SortLog<T>) {
            match event {
                SortLog::CmpInArr { .. }
                | SortLog::CmpData { .. }
                | SortLog::CmpDataU { .. }
                | SortLog::CmpAcrossArrs { .. } => self.cmps += 1,
                SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => {
                    self.element_writes += 1
                }
                SortLog::WriteDataU { .. } => self.word_writes += 1,
                _ => {}
            }
        }
    }

    #[test]
    fn it_costs_n_log_n_comparisons_and_writes_each_element_once_at_most() {
        let n = 4096;
        let mut rng = Rng(0xC057);
        let mut arr: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() { arr.swap(i, rng.below(i + 1)); }
        let mut tally = Tally::default();
        IndexHeapSort::sort(&mut arr, &mut tally);
        assert!(arr.iter().copied().eq(0..n), "not sorted");
        assert!(tally.cmps < 3 * n * 12, "{} comparisons", tally.cmps);
        assert!(tally.element_writes <= n, "{} element writes", tally.element_writes);
        assert!(tally.word_writes < 4 * n * 12, "{} word writes", tally.word_writes);
    }

    #[test]
    fn annotations_are_n_log_n_one_word_per_element_and_stable() {
        let bounds = [<IndexHeapSort as HasTimeBounds>::WORST, <IndexHeapSort as HasTimeBounds>::BEST];
        assert_eq!(bounds, [Complexity::N_LOG_N; 2]);
        assert_eq!(<IndexHeapSort as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert_eq!(<IndexHeapSort as HasSpace>::SPACE, Complexity::N1);
        assert!(<IndexHeapSort as HasStability>::STABLE);
    }
}
