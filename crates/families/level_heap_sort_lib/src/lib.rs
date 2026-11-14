//! Level heap sort, `LevelHeapSort<B, M, P, R>`: a stable heap sort through a
//! buffer of elements smaller than the array. `heap_sort_into` sorts one
//! region out of place: its elements go into the buffer, a pre-order min heap
//! is built there by selection, and the pops come back in order by
//! `sink_hole`, a hole sinking to a leaf that dies, so that no element ever
//! crosses the heap. The dead leaves are flagged in the `Marks` `M`. `select` is the stable selection, `build_levels` the
//! level ordered heap it leaves, and `sort_range` the recursion that levels a
//! range too big for the buffer. `left_size` is the heap's layout. The family
//! is declared in `level_heap.spec` and built by `level_heap.avbs`.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::keys::{extract_distinct, heap_sort_region, slide_merge, sort_region};
use array_vis_bench_traits::role::{Auxiliary, Marks, Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::{RegionKind, SortLogger};

#[cfg(test)]
pub mod test_util;

// A range this short is left to the insertion sort.
const RUN: usize = 8;

/// `B` is the buffer, carved from the array or allocated beside it, `M` the
/// marks of its dead cells, `P` the `StablePartition` the level build selects
/// with and `R` the `Rotation` it rotates with, whose scratch is the one other
/// allocation of the sort.
pub struct LevelHeapSort<B, M, P, R>(PhantomData<(B, M, P, R)>);

impl<B: Auxiliary, M: Marks, P: StablePartition, R: Rotation> LevelHeapSort<B, M, P, R> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let rotation_cells = R::scratch_size(n);
        let mut scratch = if rotation_cells > 0 { log.create_aux_arr_t(rotation_cells) } else { Vec::new() };
        let external = B::scratch_size(n);
        // A buffer cell costs the array one distinct element when the buffer is
        // carved and `M::CARVED` more when the marks are. They are carved in
        // one run, the first distinct elements of the array, as many as are
        // asked for or as there are, and the buffer has the cells they pay for:
        // the buffer's own cells first, then the marks', then any left over.
        // The rest is the range sorted. An external buffer holds nothing worth
        // keeping.
        let per_cell = usize::from(external == 0) + M::CARVED;
        let carved = extract_distinct::<R, T, U>(arr, 0, n, external.max(B::internal_size(n)) * per_cell, &mut scratch, log);
        let cells = if per_cell > 0 { carved / per_cell } else { external };
        let (front, rest) = arr.split_at_mut(carved);
        let (buffer, pairs) = front.split_at_mut(if external > 0 { 0 } else { cells });
        let mut ext = if external > 0 { log.create_aux_arr_t(external) } else { Vec::new() };
        let (heap, keeps) = if external > 0 { (&mut ext[..cells], false) } else { (buffer, true) };
        let mut marks = M::create::<T, U>(cells, log);
        if cells > 0 { log.hold_region(heap, 0, cells, RegionKind::Buffer); }
        if !pairs.is_empty() { log.hold_region(pairs, 0, pairs.len(), RegionKind::Keys); }
        sort_range::<M, P, R, T, U>(rest, 0, n - carved, heap, &mut marks, pairs, keeps, &mut scratch, log);
        if !pairs.is_empty() { log.release_region(pairs, 0, pairs.len(), RegionKind::Keys); }
        if cells > 0 { log.release_region(heap, 0, cells, RegionKind::Buffer); }
        M::free::<T, U>(&marks, log);
        if external > 0 { log.free_aux_arr_t(&ext); }
        // What was carved is distinct, so an unstable sort puts it back in
        // order, and it slides home ahead of its equals.
        if carved > 0 {
            heap_sort_region(arr, 0, carved, log);
            slide_merge::<R, T, U>(arr, 0, carved, n, &mut scratch, log);
        }
        if rotation_cells > 0 { log.free_aux_arr_t(&scratch); }
    }
}

array_vis_bench_traits::direct_sort!(LevelHeapSort<B, M, P, R>; B: Auxiliary, M: Marks, P: StablePartition, R: Rotation);

// The level build is a quickselect per boundary, quadratic at worst and linear
// expected, over `log (n / buffer)` depths; the heaps cost `n log n` on every
// input; a carved buffer adds its square to carve and slide home.
impl<B: Auxiliary, M: Marks, P: StablePartition, R: Rotation> HasTimeBounds for LevelHeapSort<B, M, P, R> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

// The buffer's cells outside the array, the marks allocated for the buffer's
// cells wherever they are, and the rotation's scratch.
impl<B: Auxiliary, M: Marks, P: StablePartition, R: Rotation + HasSpace> HasSpace for LevelHeapSort<B, M, P, R> {
    const SPACE: Complexity =
        Complexity::sum(Complexity::sum(B::MERGE_SPACE, marks_space(M::ALLOCATED_BITS, Complexity::sum(B::MERGE_SPACE, B::CARVED))), R::SPACE);
}

// The space of a mark of `bits` allocated bits per cell of a buffer of `cells`:
// nothing for a carved mark, the buffer's order for a byte, and for a bit the
// same but for a logarithm of cells, whose bits are a constant number of words.
const fn marks_space(bits: usize, cells: Complexity) -> Complexity {
    if bits == 0 || bits == 1 && Complexity::LOG_N.at_least(cells) { Complexity::CONST } else { cells }
}

// The level build keeps equals in order, each level is sorted by the stable
// heap, a level of eight or fewer by the insertion sort, what is carved is
// the first occurrence of its values, and it slides home ahead of its equals.
impl<B: Auxiliary, M: Marks, P: StablePartition, R: Rotation> HasStability for LevelHeapSort<B, M, P, R> {
    const STABLE: bool = true;
}

/// The level boundaries `2^k - 1` inside a range of `n`, ascending.
pub fn boundaries(n: usize) -> impl Iterator<Item = usize> {
    (1..).map(|k| (1 << k) - 1).take_while(move |&b| b < n)
}

/// Bring the `k` smallest elements of `arr[lo..hi]` to its front, ties to the
/// earlier, equal elements keeping their order in both parts: a quickselect over
/// the three way stable partition around a median of three, done as soon as the
/// boundary falls inside the pivot's equal group, which the stable split leaves
/// in order.
pub fn select<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    k: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    let (mut lo, mut hi, mut k) = (lo, hi, k);
    while k > 0 && k < hi - lo {
        let pivot = arr[median_of_three(arr, lo, lo + (hi - lo) / 2, hi - 1, log)];
        let (equal, above) = P::partition_three_way::<R, T, U>(arr, lo, hi, pivot, scratch, log);
        if lo + k < equal {
            hi = equal;
        } else if lo + k > above {
            k -= above - lo;
            lo = above;
        } else {
            return;
        }
    }
}

// The index of the median of three cells, compared in place.
fn median_of_three<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], a: usize, b: usize, c: usize, log: &mut U) -> usize {
    let (low, high) = if log.cmp_lt(arr, b, a) { (b, a) } else { (a, b) };
    if log.cmp_lt(arr, c, low) {
        low
    } else if log.cmp_lt(arr, c, high) {
        c
    } else {
        high
    }
}

/// The level ordered heap of `arr[lo..hi]`: the smallest `2^k - 1` elements at
/// its front for every boundary, from the largest down so that the prefixes
/// halve, equal elements in original order throughout.
pub fn build_levels<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    let mut end = hi;
    for b in boundaries(hi - lo).collect::<Vec<_>>().into_iter().rev() {
        select::<P, R, T, U>(arr, lo, end, b, scratch, log);
        end = lo + b;
    }
}

/// Sort `arr[lo..hi]` stably: a short range by insertion, a range the buffer
/// holds through the heap, and a larger one by levelling it and sorting each
/// level the same way, so a range halves with every depth until the buffer
/// holds it.
#[allow(clippy::too_many_arguments)]
pub fn sort_range<M: Marks, P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    heap: &mut [T],
    marks: &mut M::Store,
    pairs: &mut [T],
    keeps: bool,
    scratch: &mut [T],
    log: &mut U,
) {
    let m = hi - lo;
    if m <= RUN {
        sort_region(arr, lo, hi, log);
    } else if m <= heap.len() {
        heap_sort_into::<M, T, U>(arr, lo, hi, heap, marks, pairs, keeps, log);
    } else {
        build_levels::<P, R, T, U>(arr, lo, hi, scratch, log);
        let mut start = lo;
        for b in boundaries(m).map(|b| lo + b).chain([hi]) {
            sort_range::<M, P, R, T, U>(arr, start, b, heap, marks, pairs, keeps, scratch, log);
            start = b;
        }
    }
}

/// Size of the left subtree of a complete binary tree of `m` nodes: the full
/// levels above the last one are split evenly, and the last level fills the
/// left half first.
pub fn left_size(m: usize) -> usize {
    if m < 2 { return 0; }
    let height = usize::BITS - 1 - m.leading_zeros();
    let half = 1 << (height - 1);
    let last = m - ((1 << height) - 1);
    half - 1 + last.min(half)
}

/// Sort `arr[lo..hi]` stably through `heap`, a buffer of at least as many
/// cells, and as many of the marks `M`, clear coming in and clear again going
/// out, `pairs` being the region they are carved from if they are. The
/// elements go into the buffer's first cells, a pre-order min heap is built
/// there by selection, and the pops come
/// back into `arr[lo..hi]` in order: each pop takes the root and sinks the
/// hole it leaves to a leaf, which dies. Stable because the build leaves
/// equals in pre-order, a rising left child moves one cell, a rising right
/// child passes only the left subtree and only when strictly smaller than its
/// root, and the root is the first of the minima. A buffer that `keeps` its
/// elements, one carved from the array, is swapped with rather than written,
/// and each dead leaf takes the buffer element the output cell held, so it
/// keeps them as a set, scrambled; a buffer that does not is never read before
/// it is written.
#[allow(clippy::too_many_arguments)]
pub fn heap_sort_into<M: Marks, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    heap: &mut [T],
    marks: &mut M::Store,
    pairs: &mut [T],
    keeps: bool,
    log: &mut U,
) {
    let m = hi - lo;
    for i in 0..m {
        if keeps { swap_across(arr, lo + i, heap, i, log) } else { log.write_accross(arr, lo + i, heap, i) }
    }
    build_heap(heap, 0, m, log);
    for out in lo..hi {
        let least = heap[0];
        let grave = sink_hole::<M, T, U>(heap, m, marks, pairs, log);
        let held = arr[out];
        log.write_data(arr, out, least);
        if keeps { log.write_data(heap, grave, held); }
    }
    // Each pop killed one cell and none twice, so all `m` are marked.
    for i in 0..m { M::unmark(marks, pairs, i, log); }
}

/// Make `heap[p..p + m]` a min heap in pre-order: the first minimum of the
/// segment moves to `p`, past elements all above it, and the two child
/// segments follow. Equal elements keep their order, as they do not cross.
pub fn build_heap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(heap: &mut [T], p: usize, m: usize, log: &mut U) {
    if m < 2 { return; }
    let best = (p + 1..p + m).fold(p, |best, i| if log.cmp_lt(heap, i, best) { i } else { best });
    if best > p {
        let x = heap[best];
        log.shift_insert(heap, best, p, x);
    }
    let left = left_size(m);
    build_heap(heap, p + 1, left, log);
    build_heap(heap, p + 1 + left, m - 1 - left, log);
}

// The hole opened at the root sinks over the live cells of the tree of `m`,
// pulling the smaller live child up, ties to the left, until no live child is
// below it; the cell it stops in dies and is returned. A dead cell's subtree
// is dead throughout, so a right child rising over a dead left child passes
// nothing live.
fn sink_hole<M: Marks, T: Ord + Copy, U: ?Sized + SortLogger<T>>(heap: &mut [T], m: usize, marks: &mut M::Store, pairs: &mut [T], log: &mut U) -> usize {
    let (mut p, mut size) = (0, m);
    loop {
        let left = left_size(size);
        let right = size - 1 - left;
        let (l, r) = (p + 1, p + 1 + left);
        let l_live = left > 0 && !M::is_marked(marks, pairs, l, log);
        let r_live = right > 0 && !M::is_marked(marks, pairs, r, log);
        let (c, csize) = match (l_live, r_live) {
            (false, false) => {
                M::mark(marks, pairs, p, log);
                return p;
            }
            (true, false) => (l, left),
            (false, true) => (r, right),
            (true, true) => if log.cmp_lt(heap, r, l) { (r, right) } else { (l, left) },
        };
        log.write(heap, p, c);
        (p, size) = (c, csize);
    }
}

fn swap_across<T: Ord + Copy, U: ?Sized + SortLogger<T>>(a: &mut [T], i: usize, b: &mut [T], j: usize, log: &mut U) {
    let (x, y) = (a[i], b[j]);
    log.write_data(a, i, y);
    log.write_data(b, j, x);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{items, pairs, Item, Replay, Rng, Tally};
    use array_vis_bench_traits::DirectSort;
    use extraction_lib::RotationPartition;
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use scratchspace_lib::{BitMarks, ByteMarks, ExternalBuffer, Fixed, Half, InArrayBuffer, InArrayMarks, Log, Sixteenth, Sqrt, TenLog, TwoSqrt};
    use sort_logger::{NoOpLogger, SortLog};
    use std::collections::{HashMap, HashSet};

    // The shapes under test mark in bytes, rotate by reversal and partition by
    // rotation unless a test names others.
    type Level<B, M = ByteMarks> = LevelHeapSort<B, M, RotationPartition, ReversalRotation>;

    // The input shapes: every key repeated often, so that ties are the rule.
    fn input(kind: usize, n: usize, rng: &mut Rng) -> Vec<Item> {
        let keys: Vec<u32> = match kind {
            0 => (0..n).map(|_| rng.next(n.max(1) * 4) as u32).collect(),
            1 => (0..n as u32).collect(),
            2 => (0..n as u32).rev().collect(),
            3 => vec![5; n],
            4 => (0..n).map(|i| (i % 7) as u32).collect(),
            5 => (0..n).map(|_| rng.next(2) as u32).collect(),
            6 => (0..n).map(|_| rng.next(10) as u32).collect(),
            _ => (0..n).map(|_| rng.next(40) as u32).collect(),
        };
        items(&keys)
    }
    const KINDS: usize = 8;

    // Buffer cells carry tags from 10000 up, so they are told from the region's.
    fn buffer(m: usize) -> Vec<Item> { (0..m).map(|i| Item { key: 3, tag: 10_000 + i as u32 }).collect() }

    #[test]
    fn the_left_subtree_of_a_complete_tree_is_the_bigger_half_of_the_last_level() {
        assert_eq!((1..=8).map(left_size).collect::<Vec<_>>(), [0, 1, 1, 2, 3, 3, 3, 4]);
    }

    // The region a filler carves its marks from: distinct, sorted, tagged
    // from 20000 up.
    fn pairs_of<M: Marks>(cells: usize) -> Vec<Item> { (0..cells * M::CARVED).map(|i| Item { key: i as u32, tag: 20_000 + i as u32 }).collect() }

    fn a_region_sorts_stably<M: Marks>() {
        let mut rng = Rng(0x0EA9);
        for n in [1usize, 2, 3, 4, 7, 8, 9, 31, 32, 33, 100, 257, 600, 1500] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let lo = rng.next(3).min(n);
                let hi = n - rng.next(3).min(n - lo);
                let m = hi - lo;
                let mut arr = arr0.clone();
                let mut heap = buffer(m + rng.next(3));
                let mut marks = M::create::<Item, _>(heap.len(), &mut NoOpLogger);
                let mut carved = pairs_of::<M>(heap.len());
                heap_sort_into::<M, _, _>(&mut arr, lo, hi, &mut heap, &mut marks, &mut carved, true, &mut NoOpLogger);
                let mut want = arr0.clone();
                want[lo..hi].sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "{}: kind {kind}, n={n}, {lo}..{hi}", M::NAME);
                let mut tags: Vec<u32> = heap.iter().map(|x| x.tag).collect();
                tags.sort();
                assert_eq!(tags, (0..heap.len() as u32).map(|i| 10_000 + i).collect::<Vec<_>>(), "{}: kind {kind}, n={n}: the buffer lost an element", M::NAME);
                assert!((0..heap.len()).all(|i| !M::is_marked(&marks, &carved, i, &mut NoOpLogger)), "{}: kind {kind}, n={n}: a mark is left set", M::NAME);
                assert_eq!(pairs(&carved), pairs(&pairs_of::<M>(heap.len())), "{}: kind {kind}, n={n}: the carved marks are not as they were", M::NAME);
            }
        }
    }

    #[test]
    fn a_region_sorts_stably_through_the_buffer_which_keeps_its_elements_under_any_marks() {
        a_region_sorts_stably::<ByteMarks>();
        a_region_sorts_stably::<BitMarks>();
        a_region_sorts_stably::<InArrayMarks>();
    }

    // A buffer that is not kept is only ever read where it was written: the
    // replay's copy of it starts blank, so a read of an unwritten cell would
    // leave the two sorts apart.
    #[test]
    fn a_buffer_not_kept_is_written_before_it_is_read() {
        let mut rng = Rng(0xB0FF);
        for n in [9usize, 40, 257, 1000] {
            let mut arr: Vec<usize> = (0..n).map(|_| 1 + rng.next(n)).collect();
            let mut replay = Replay::of(&arr);
            let mut heap: Vec<usize> = (0..n).map(|_| usize::MAX).collect();
            replay.log_aux_arr_t(&heap);
            let mut dead = replay.create_aux_arr_u8(n);
            heap_sort_into::<ByteMarks, _, _>(&mut arr, 0, n, &mut heap, &mut dead, &mut [], false, &mut replay);
            assert_eq!(replay.main(), &arr[..], "n={n}: an unwritten cell was read");
        }
    }

    #[test]
    fn no_move_goes_unlogged() {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                let mut heap = replay.create_aux_arr_t(n);
                let mut dead = replay.create_aux_arr_u8(n);
                heap_sort_into::<ByteMarks, _, _>(&mut arr, 0, n, &mut heap, &mut dead, &mut [], false, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "a move went unlogged, n={n} values={values}");
            }
        }
    }

    // The tags of the `k` smallest of `arr`, ties to the earlier.
    fn smallest(arr: &[Item], k: usize) -> HashSet<u32> {
        let mut ranked = arr.to_vec();
        ranked.sort_by_key(|x| x.key);
        ranked[..k].iter().map(|x| x.tag).collect()
    }

    // Equal keys appear in original order: a partition keeps the order within
    // each of its classes and a selection stops in an equal group.
    fn equals_in_order(arr: &[Item]) -> bool {
        let mut last = HashMap::new();
        arr.iter().all(|x| last.insert(x.key, x.tag).is_none_or(|t| t < x.tag))
    }

    #[test]
    fn the_levels_are_rank_intervals_with_equals_in_order() {
        let mut rng = Rng(0x1E7E1);
        for n in [1usize, 2, 3, 7, 8, 15, 16, 100, 257, 1000] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                build_levels::<RotationPartition, ReversalRotation, _, _>(&mut arr, 0, n, &mut [], &mut NoOpLogger);
                for hi in boundaries(n) {
                    let got: HashSet<u32> = arr[..hi].iter().map(|x| x.tag).collect();
                    assert_eq!(got, smallest(&arr0, hi), "kind {kind}, n={n}: the first {hi}");
                    assert!(equals_in_order(&arr[..hi]) && equals_in_order(&arr[hi..]), "kind {kind}, n={n}: equals out of order at {hi}");
                }
            }
        }
    }

    fn sorts_stably<S: DirectSort>(name: &str) {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 9, 31, 32, 33, 64, 100, 257, 600, 1500, 3001] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                S::sort(&mut arr, &mut NoOpLogger);
                let mut want = arr0.clone();
                want.sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "{name}: kind {kind}, n={n}");
            }
        }
    }

    #[test]
    fn every_buffer_sorts_stably() {
        sorts_stably::<Level<InArrayBuffer<Sqrt>>>("in array sqrt");
        sorts_stably::<Level<InArrayBuffer<TwoSqrt>>>("in array 2 sqrt");
        sorts_stably::<Level<InArrayBuffer<Log>>>("in array log");
        sorts_stably::<Level<InArrayBuffer<Fixed<512>>>>("in array 512");
        sorts_stably::<Level<InArrayBuffer<Half>>>("in array half");
        sorts_stably::<Level<InArrayBuffer<Sixteenth>>>("in array sixteenth");
        sorts_stably::<Level<ExternalBuffer<Sqrt>>>("external sqrt");
        sorts_stably::<Level<ExternalBuffer<Log>>>("external log");
        sorts_stably::<Level<ExternalBuffer<Half>>>("external half");
    }

    // Every rotation of the sort is `R`'s; trinity and the auxiliary rotation
    // run through the scratch the sort allocates once.
    #[test]
    fn any_rotation_sorts_stably() {
        sorts_stably::<LevelHeapSort<InArrayBuffer<Sqrt>, ByteMarks, RotationPartition, TrinityRotation>>("in array sqrt, trinity");
        sorts_stably::<LevelHeapSort<ExternalBuffer<Sqrt>, InArrayMarks, RotationPartition, AuxiliaryRotation>>("external sqrt, in array marks, auxiliary");
    }

    // The inputs of few values leave a carved shape short of what it asked
    // for, down to no buffer cell at all.
    #[test]
    fn any_marks_sort_stably_in_the_array_or_beside_it() {
        sorts_stably::<Level<InArrayBuffer<Sqrt>, BitMarks>>("in array sqrt, bits");
        sorts_stably::<Level<ExternalBuffer<Half>, BitMarks>>("external half, bits");
        sorts_stably::<Level<InArrayBuffer<Sqrt>, InArrayMarks>>("in array sqrt, in array marks");
        sorts_stably::<Level<InArrayBuffer<TenLog>, InArrayMarks>>("in array 10 log, in array marks");
        sorts_stably::<Level<InArrayBuffer<Half>, InArrayMarks>>("in array half, in array marks");
        sorts_stably::<Level<ExternalBuffer<Sqrt>, InArrayMarks>>("external sqrt, in array marks");
        sorts_stably::<Level<ExternalBuffer<Half>, InArrayMarks>>("external half, in array marks");
    }

    // Counts the arrays a sort allocates through the logger.
    #[derive(Default)]
    struct Allocations(usize);
    impl SortLogger<usize> for Allocations {
        fn log(&mut self, event: SortLog<usize>) {
            if matches!(event, SortLog::CreateAuxArr { .. } | SortLog::CreateAuxArrT { .. }) { self.0 += 1; }
        }
    }

    fn allocations<S: DirectSort>() -> usize {
        let mut rng = Rng(0xA110C);
        let mut arr: Vec<usize> = (0..3000).map(|_| rng.next(1 << 20)).collect();
        let mut seen = Allocations::default();
        S::sort(&mut arr, &mut seen);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        seen.0
    }

    #[test]
    fn a_carved_buffer_with_carved_marks_allocates_nothing() {
        assert_eq!(allocations::<Level<InArrayBuffer<TenLog>, InArrayMarks>>(), 0);
        assert_eq!(allocations::<Level<InArrayBuffer<TenLog>, BitMarks>>(), 1);
        assert_eq!(allocations::<Level<InArrayBuffer<TenLog>>>(), 1);
        assert_eq!(allocations::<Level<ExternalBuffer<TenLog>, InArrayMarks>>(), 1);
        assert_eq!(allocations::<Level<ExternalBuffer<TenLog>>>(), 2);
    }

    fn moves_are_all_logged<S: DirectSort>(name: &str) {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                S::sort(&mut arr, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{name}: not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "{name}: a move went unlogged, n={n} values={values}");
            }
        }
    }

    #[test]
    fn no_shape_moves_an_element_without_logging_it() {
        moves_are_all_logged::<Level<InArrayBuffer<Sqrt>>>("in array sqrt");
        moves_are_all_logged::<Level<ExternalBuffer<Log>>>("external log");
        moves_are_all_logged::<LevelHeapSort<ExternalBuffer<Sqrt>, ByteMarks, RotationPartition, TrinityRotation>>("external sqrt, trinity");
        moves_are_all_logged::<Level<InArrayBuffer<Sqrt>, InArrayMarks>>("in array sqrt, in array marks");
        moves_are_all_logged::<Level<ExternalBuffer<Sqrt>, InArrayMarks>>("external sqrt, in array marks");
        moves_are_all_logged::<Level<InArrayBuffer<Sqrt>, BitMarks>>("in array sqrt, bits");
    }

    #[test]
    fn the_bounds_follow_from_the_buffer_and_the_rotation() {
        assert_eq!(<Level<InArrayBuffer<Sqrt>> as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<Level<InArrayBuffer<Sqrt>> as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert_eq!(<Level<InArrayBuffer<Sqrt>> as HasSpace>::SPACE, Complexity::SQRT_N);
        assert_eq!(<Level<ExternalBuffer<Half>> as HasSpace>::SPACE, Complexity::N1);
        assert!(<Level<InArrayBuffer<Log>> as HasSpace>::SPACE.is_in_place());
        assert!(<Level<InArrayBuffer<Sqrt>> as HasStability>::STABLE);
    }

    // A byte per cell is the buffer's order wherever the buffer is, a bit per
    // cell the same but for a logarithm of them, and a carved mark is nothing.
    #[test]
    fn the_space_follows_from_the_marks() {
        assert_eq!(<Level<InArrayBuffer<Log>> as HasSpace>::SPACE, Complexity::LOG_N);
        assert_eq!(<Level<InArrayBuffer<Log>, BitMarks> as HasSpace>::SPACE, Complexity::CONST);
        assert_eq!(<Level<InArrayBuffer<Sqrt>, BitMarks> as HasSpace>::SPACE, Complexity::SQRT_N);
        assert_eq!(<Level<InArrayBuffer<Sqrt>, InArrayMarks> as HasSpace>::SPACE, Complexity::CONST);
        assert_eq!(<Level<InArrayBuffer<Half>, InArrayMarks> as HasSpace>::SPACE, Complexity::CONST);
        assert_eq!(<Level<ExternalBuffer<Sqrt>, InArrayMarks> as HasSpace>::SPACE, Complexity::SQRT_N);
    }

    // The build is `m log m` comparisons and up to as many moves, the pops
    // one comparison per level of the sink, so a few times `m log m` bounds both.
    #[test]
    fn costs_m_log_m_comparisons_and_moves() {
        let m = 4096;
        let mut rng = Rng(0xC0FFEE);
        let mut arr: Vec<usize> = (0..m).map(|_| rng.next(1 << 30)).collect();
        let (mut heap, mut dead) = (vec![0usize; m], vec![0u8; m]);
        let mut tally = Tally::default();
        heap_sort_into::<ByteMarks, _, _>(&mut arr, 0, m, &mut heap, &mut dead, &mut [], false, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 3 * m * 12, "comparisons {}", tally.cmps);
        assert!(tally.moves < 3 * m * 12, "moves {}", tally.moves);
    }
}
