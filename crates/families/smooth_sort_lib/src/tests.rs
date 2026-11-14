//! The tests of every combination of direction, shape, sift and build: that
//! each sorts every kind of input, that no move goes unlogged, and that the
//! forest keeps its two orders, a heap in each tree and the roots ascending,
//! through the build and through every removal.

use array_vis_bench_traits::DirectSort;
use heap_sort_lib::direction::{Direction, MaxForward, MinReverse};
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use sort_logger::{NoOpLogger, SortLog, SortLogger};

use crate::heap::SmoothHeap;
use crate::tree::{above, children};
use crate::{Build, BottomUp, Forest, Heapify, Insert, LazyInsert, Leonardo, Pairs, Perfect, Shape, Sift, SmoothSort};

// Calls `$check::<SmoothSort<D, S, F, B>>(name)` for each of the 24 types.
macro_rules! every_sort {
    ($check:ident) => {
        every_sort!(@dir $check; MinReverse);
        every_sort!(@dir $check; MaxForward);
    };
    (@dir $check:ident; $d:ty) => {
        every_sort!(@shape $check; $d, Leonardo);
        every_sort!(@shape $check; $d, Perfect);
    };
    (@shape $check:ident; $d:ty, $s:ty) => {
        every_sort!(@sift $check; $d, $s, Pairs);
        every_sort!(@sift $check; $d, $s, BottomUp);
    };
    (@sift $check:ident; $d:ty, $s:ty, $f:ty) => {
        every_sort!(@build $check; $d, $s, $f, Insert);
        every_sort!(@build $check; $d, $s, $f, LazyInsert);
        every_sort!(@build $check; $d, $s, $f, Heapify);
    };
    (@build $check:ident; $d:ty, $s:ty, $f:ty, $b:ty) => {
        $check::<SmoothSort<$d, $s, $f, $b>, $d, $s, $f, $b>(stringify!($d, $s, $f, $b));
    };
}

struct Rng(u64);
impl Rng {
    fn next(&mut self, m: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % m
    }
}

// Inputs of every shape a heap sort is told apart on, `n` long.
fn inputs(n: usize) -> Vec<Vec<usize>> {
    let mut rng = Rng(n as u64 + 1);
    vec![
        (0..n).map(|_| rng.next(n * 4 + 1)).collect(),
        (0..n).map(|_| rng.next(4)).collect(),
        (0..n).collect(),
        (0..n).rev().collect(),
        vec![7; n],
        (0..n).map(|i| i.min(n - 1 - i)).collect(),
        (0..n).map(|i| i % 9).collect(),
    ]
}

const SIZES: [usize; 22] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 31, 32, 33, 64, 100, 255, 1000, 3000];

fn sorts_every_input<A: DirectSort, D, S, F, B>(name: &str) {
    for n in SIZES {
        for input in inputs(n) {
            let mut sorted = input.clone();
            sorted.sort();
            let mut arr = input.clone();
            A::sort(&mut arr, &mut NoOpLogger);
            assert_eq!(arr, sorted, "{name} on {input:?}");
        }
    }
}

#[test]
fn every_combination_sorts_every_input() { every_sort!(sorts_every_input); }

// Replays the writes onto a copy of the array, so that comparing the copy with
// the array shows that no move went unlogged.
struct Replay(Vec<usize>);
impl SortLogger<usize> for Replay {
    fn log(&mut self, event: SortLog<usize>) {
        match event {
            SortLog::Swap { ind_a, ind_b, .. } => self.0.swap(ind_a, ind_b),
            SortLog::WriteInArr { ind_a, ind_b, .. } => self.0[ind_a] = self.0[ind_b],
            SortLog::WriteData { ind, data, .. } => self.0[ind] = data,
            _ => {}
        }
    }
}

fn logs_every_move<A: DirectSort, D, S, F, B>(name: &str) {
    for n in [10, 100, 1000] {
        for input in inputs(n) {
            let mut arr = input.clone();
            let mut replay = Replay(input);
            A::sort(&mut arr, &mut replay);
            assert_eq!(replay.0, arr, "{name} moved something it did not log");
        }
    }
}

#[test]
fn no_combination_moves_an_element_without_logging_it() { every_sort!(logs_every_move); }

// Whether every tree of the forest is a heap and the roots ascend, the last
// outranking all, over the first `len` cells.
fn is_smooth<D: Direction, S: Shape>(arr: &[usize], forest: &Forest, len: usize) -> bool {
    fn heap<D: Direction, S: Shape>(arr: &[usize], root: usize, order: usize) -> bool {
        children::<S>(root, order).map_or(true, |pair| {
            pair.iter().all(|&(child, child_order)| !above::<D, usize, NoOpLogger>(arr, child, root, &mut NoOpLogger) && heap::<D, S>(arr, child, child_order))
        })
    }
    let mut end = 0;
    let roots: Vec<(usize, usize)> = forest.orders().iter().map(|&order| { end += S::size(order); (end - 1, order) }).collect();
    end == len
        && roots.iter().all(|&(root, order)| heap::<D, S>(arr, root, order))
        && roots.windows(2).all(|pair| !above::<D, usize, NoOpLogger>(arr, pair[0].0, pair[1].0, &mut NoOpLogger))
}

fn keeps_the_forest<A, D: Direction, S: Shape, F: Sift, B: Build>(name: &str) {
    for n in [1, 2, 3, 4, 5, 8, 9, 14, 15, 16, 41, 100, 200] {
        for input in inputs(n) {
            let mut arr = input;
            let mut forest = Forest::default();
            B::build::<D, S, F, usize, NoOpLogger>(&mut arr, &mut forest, &mut NoOpLogger);
            assert!(is_smooth::<D, S>(&arr, &forest, n), "{name}: the build of {arr:?}");
            for len in (1..n).rev() {
                SmoothHeap::<D, S, F>::remove_last(&mut arr, &mut forest, len + 1, &mut NoOpLogger);
                assert!(is_smooth::<D, S>(&arr, &forest, len), "{name}: {len} left of {arr:?}");
            }
        }
    }
}

#[test]
fn the_forest_holds_through_the_build_and_every_removal() { every_sort!(keeps_the_forest); }

#[test]
fn the_heap_algorithm_names_the_last_cell_the_root() {
    assert_eq!(<SmoothSort<MaxForward, Leonardo, Pairs, Insert> as HeapAlgorithm>::root_phys(10), 9);
    assert_eq!(<SmoothSort<MinReverse, Leonardo, Pairs, Insert> as HeapAlgorithm>::root_phys(10), 0);
}

// Counts the comparisons and the moves.
#[derive(Default)]
struct Tally { cmps: usize, moves: usize }
impl SortLogger<usize> for Tally {
    fn log(&mut self, event: SortLog<usize>) {
        match event {
            SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => self.moves += 1,
            SortLog::CmpInArr { .. } | SortLog::CmpData { .. } => self.cmps += 1,
            _ => {}
        }
    }
}

const COUNTED: usize = 1 << 14;

fn counted<A: DirectSort>(input: Vec<usize>) -> Tally {
    let mut arr = input;
    let mut tally = Tally::default();
    A::sort(&mut arr, &mut tally);
    tally
}

fn random_input() -> Vec<usize> {
    let mut rng = Rng(5);
    (0..COUNTED).map(|_| rng.next(1 << 30)).collect()
}

fn costs_little_on_a_sorted_input<A: DirectSort, D, S, F, B>(name: &str) {
    let tally = counted::<A>((0..COUNTED).collect());
    assert!(tally.cmps <= 5 * COUNTED, "{name}: {} comparisons for {COUNTED} sorted elements", tally.cmps);
    assert_eq!(tally.moves, 0, "{name} moved elements already in place");
}

#[test]
fn a_sorted_input_is_linear_and_moves_nothing() { every_sort!(costs_little_on_a_sorted_input); }

fn costs_n_log_n_on_a_random_input<A: DirectSort, D, S, F, B>(name: &str) {
    let tally = counted::<A>(random_input());
    let n_log_n = COUNTED * COUNTED.ilog2() as usize;
    assert!(tally.cmps <= 4 * n_log_n, "{name}: {} comparisons", tally.cmps);
    assert!(tally.moves <= 2 * n_log_n, "{name}: {} moves", tally.moves);
}

#[test]
fn a_random_input_is_n_log_n() { every_sort!(costs_n_log_n_on_a_random_input); }

fn comparisons<A: DirectSort>() -> usize { counted::<A>(random_input()).cmps }

#[test]
fn the_axes_move_the_comparisons_the_way_they_are_meant_to() {
    type Max<S, F, B> = SmoothSort<MaxForward, S, F, B>;
    // The bottom up sift, one comparison a level on the way down.
    assert!(comparisons::<Max<Leonardo, BottomUp, Insert>>() < comparisons::<Max<Leonardo, Pairs, Insert>>());
    assert!(comparisons::<Max<Perfect, BottomUp, Heapify>>() < comparisons::<Max<Perfect, Pairs, Heapify>>());
    // The build that rectifies the roots least often, down to once a tree.
    assert!(comparisons::<Max<Leonardo, Pairs, Heapify>>() < comparisons::<Max<Leonardo, Pairs, LazyInsert>>());
    assert!(comparisons::<Max<Leonardo, Pairs, LazyInsert>>() < comparisons::<Max<Leonardo, Pairs, Insert>>());
    // The Leonardo trees, a root over uneven subtrees and fewer trees to walk.
    assert!(comparisons::<Max<Leonardo, Pairs, Insert>>() < comparisons::<Max<Perfect, Pairs, Insert>>());
}

// A reversed heap direction sorts the same elements the same way, the min heap
// laid out from the right, so the two directions cost the same to a few percent.
#[test]
fn the_two_directions_cost_the_same() {
    let max = comparisons::<SmoothSort<MaxForward, Leonardo, Pairs, Insert>>() as f64;
    let min = comparisons::<SmoothSort<MinReverse, Leonardo, Pairs, Insert>>() as f64;
    assert!((max - min).abs() < 0.01 * max, "{max} against {min}");
}
