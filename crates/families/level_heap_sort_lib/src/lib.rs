//! Level heap sort, `LevelHeapSort`: a stable heap sort through a buffer of
//! elements. `heap_sort_into` sorts one region out of place: its elements go
//! into the buffer, a pre-order min heap is built there by selection, and the
//! pops come back in order by `sink_hole`, a hole sinking to a leaf that dies,
//! so that no element ever crosses the heap. `left_size` is the layout.

use sort_logger::SortLogger;

#[cfg(test)]
pub mod test_util;

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
/// cells, and `dead`, as many marks. The elements are swapped into the
/// buffer's first cells, a pre-order min heap is built there by selection, and
/// the pops come back into `arr[lo..hi]` in order: each pop takes the root and
/// sinks the hole it leaves to a leaf, which dies and takes the buffer element
/// the output cell held. Stable because the build leaves equals in pre-order,
/// a rising left child moves one cell, a rising right child passes only the
/// left subtree and only when strictly smaller than its root, and the root is
/// the first of the minima. The buffer keeps its elements as a set, scrambled.
pub fn heap_sort_into<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    heap: &mut [T],
    dead: &mut [u8],
    log: &mut U,
) {
    let m = hi - lo;
    for i in 0..m { swap_across(arr, lo + i, heap, i, log); }
    build_heap(heap, 0, m, log);
    for i in 0..m { log.write_data_u8(dead, i, 0); }
    for out in lo..hi {
        let least = heap[0];
        let grave = sink_hole(heap, m, dead, log);
        let held = arr[out];
        log.write_data(arr, out, least);
        log.write_data(heap, grave, held);
    }
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
fn sink_hole<T: Ord + Copy, U: ?Sized + SortLogger<T>>(heap: &mut [T], m: usize, dead: &mut [u8], log: &mut U) -> usize {
    let (mut p, mut size) = (0, m);
    loop {
        let left = left_size(size);
        let right = size - 1 - left;
        let (l, r) = (p + 1, p + 1 + left);
        let (l_live, r_live) = (left > 0 && dead[l] == 0, right > 0 && dead[r] == 0);
        let (c, csize) = match (l_live, r_live) {
            (false, false) => {
                log.write_data_u8(dead, p, 1);
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
    use sort_logger::NoOpLogger;

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

    #[test]
    fn a_region_sorts_stably_through_the_buffer_which_keeps_its_elements() {
        let mut rng = Rng(0x0EA9);
        for n in [1usize, 2, 3, 4, 7, 8, 9, 31, 32, 33, 100, 257, 600, 1500] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let lo = rng.next(3).min(n);
                let hi = n - rng.next(3).min(n - lo);
                let m = hi - lo;
                let mut arr = arr0.clone();
                let mut heap = buffer(m + rng.next(3));
                let mut dead = vec![7u8; heap.len()];
                heap_sort_into(&mut arr, lo, hi, &mut heap, &mut dead, &mut NoOpLogger);
                let mut want = arr0.clone();
                want[lo..hi].sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "kind {kind}, n={n}, {lo}..{hi}");
                let mut tags: Vec<u32> = heap.iter().map(|x| x.tag).collect();
                tags.sort();
                assert_eq!(tags, (0..heap.len() as u32).map(|i| 10_000 + i).collect::<Vec<_>>(), "kind {kind}, n={n}: the buffer lost an element");
                assert!(dead[..m].iter().all(|&d| d == 1) && dead[m..].iter().all(|&d| d == 7), "kind {kind}, n={n}: the marks are off");
            }
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
                heap_sort_into(&mut arr, 0, n, &mut heap, &mut dead, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "a move went unlogged, n={n} values={values}");
            }
        }
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
        heap_sort_into(&mut arr, 0, m, &mut heap, &mut dead, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 3 * m * 12, "comparisons {}", tally.cmps);
        assert!(tally.moves < 3 * m * 12, "moves {}", tally.moves);
    }
}
