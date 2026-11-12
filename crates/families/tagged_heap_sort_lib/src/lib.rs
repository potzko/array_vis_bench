//! Tagged heap sort, `TaggedHeapSort`: a stable heap sort in the pre-order
//! layout of a complete binary tree, built by selection so that equal elements
//! keep their order as their cells, with a tag only for the element a pop
//! moves from the array's end to the root. `left_size` is the layout, `build`
//! the selection build, `sift_down` the tagged sift, `smaller` the order the
//! heap keeps. The family is declared in `tagged_heap.spec` and built by
//! `tagged_heap.avbs`.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

#[cfg(test)]
pub mod test_util;

/// The jumpers' stamps by cell. A jumper is the element a pop moved from the
/// array's end to the root: it was the last of its untagged equals, so it is
/// later than all of them, and among jumpers of one value a newer stamp is the
/// earlier element, since the older jumper was the latest when the newer one
/// was still untagged.
type Tags = HashMap<usize, usize, BuildHasherDefault<CellHasher>>;

// A cell is one word, so one multiplication spreads it, folded so that the low
// bits the table indexes by carry the high bits' mixing.
#[derive(Default)]
pub struct CellHasher(u64);
impl Hasher for CellHasher {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, bytes: &[u8]) { for &b in bytes { self.write_u64(b as u64) } }
    fn write_usize(&mut self, cell: usize) { self.write_u64(cell as u64) }
    fn write_u64(&mut self, cell: u64) {
        let spread = (self.0 ^ cell).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.0 = spread ^ (spread >> 32);
    }
}

/// Stable, `O(n log n)`, with a map holding a stamp per element that has jumped.
pub struct TaggedHeapSort;

impl TaggedHeapSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        run(arr, log, |_, _, _| {});
    }
}

// The sort, with `after_pop` seeing the array, the live count and the tags
// after each pop, for the tests; `sort` hands it nothing to do.
fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: FnMut(&[T], usize, &Tags)>(arr: &mut [T], log: &mut U, mut after_pop: F) {
    let n = arr.len();
    if n < 2 { return; }
    build(arr, 0, n, log);
    let mut tags = Tags::default();
    let mut stamp = 0;
    // The root is the earliest of the minima, see `smaller`, and goes to the
    // end; the element from the end becomes a jumper, keeping an old stamp or
    // taking a new one, and sinks from the root over the live cells.
    for end in (1..n).rev() {
        log.swap(arr, 0, end);
        tags.remove(&0);
        let tag = tags.remove(&end).unwrap_or_else(|| {
            stamp += 1;
            stamp
        });
        sift_down(arr, n, end, tag, &mut tags, log);
        after_pop(arr, end, &tags);
    }
    log.reverse(arr);
}

array_vis_bench_traits::direct_sort!(TaggedHeapSort);

// The build selects a minimum per node, `n` comparisons for the root and
// halving below it, and the pops sift `log n` deep, on every input.
impl HasTimeBounds for TaggedHeapSort {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

// The map holds the jumpers still in the heap, a fraction of `n` on random
// data and all of it at worst.
impl HasSpace for TaggedHeapSort {
    const SPACE: Complexity = Complexity::N1;
}

// Untagged equals keep their order as their cells: the build moves a minimum
// only past elements above it, a rising child passes nothing equal, and the
// one element that crosses the heap carries its order as a stamp.
impl HasStability for TaggedHeapSort {
    const STABLE: bool = true;
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

/// Make `arr[p..p + m]` a min heap in pre-order: the first minimum of the
/// segment moves to `p`, past elements all above it, and the two child
/// segments follow. Equal elements keep their order, as they do not cross.
pub fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], p: usize, m: usize, log: &mut U) {
    if m < 2 { return; }
    let best = (p + 1..p + m).fold(p, |best, i| if log.cmp_lt(arr, i, best) { i } else { best });
    if best > p {
        let x = arr[best];
        log.shift_insert(arr, best, p, x);
    }
    let left = left_size(m);
    build(arr, p + 1, left, log);
    build(arr, p + 1 + left, m - 1 - left, log);
}

// Sink the jumper of stamp `tag` from the root over the live cells `arr[..end]`
// of the tree of `n`, and record its stamp at the cell it stops in. A tie
// between the children goes left, an adjacent cell; the right child rises over
// the left subtree only when strictly smaller, or when the left child is a
// tagged equal, whose subtree holds no untagged equal of it. Only a tagged
// rising child touches the map on the way.
fn sift_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], n: usize, end: usize, tag: usize, tags: &mut Tags, log: &mut U) {
    let (mut p, mut m) = (0, n);
    loop {
        let left = left_size(m);
        let l = p + 1;
        if left == 0 || l >= end { break; }
        let r = l + left;
        let right = m - 1 - left;
        let take_right = right > 0 && r < end && smaller(arr, r, l, tags, log);
        let (c, cm) = if take_right { (r, right) } else { (l, left) };
        if !smaller_than_jumper(arr, c, p, tag, tags, log) { break; }
        log.swap(arr, p, c);
        if let Some(t) = tags.remove(&c) { tags.insert(p, t); }
        (p, m) = (c, cm);
    }
    tags.insert(p, tag);
}

// The order the heap keeps between two cells: by value, then by `tie`.
fn smaller<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], a: usize, b: usize, tags: &Tags, log: &mut U) -> bool {
    if log.cmp_lt(arr, a, b) { return true; }
    if log.cmp_lt(arr, b, a) { return false; }
    tie(a, tags.get(&a).copied(), b, tags.get(&b).copied())
}

// The same order between the cell `c` and the jumper of stamp `tag` at `p`,
// whose stamp is not in the map while it sinks.
fn smaller_than_jumper<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], c: usize, p: usize, tag: usize, tags: &Tags, log: &mut U) -> bool {
    if log.cmp_lt(arr, c, p) { return true; }
    if log.cmp_lt(arr, p, c) { return false; }
    tie(c, tags.get(&c).copied(), p, Some(tag))
}

// Equal values: an untagged element before a tagged one, untagged ones by cell,
// which is their original order, and tagged ones by stamp, the newer first.
fn tie(a: usize, tag_a: Option<usize>, b: usize, tag_b: Option<usize>) -> bool {
    match (tag_a, tag_b) {
        (None, None) => a < b,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(sa), Some(sb)) => sa > sb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{items, pairs, Item, Replay, Rng, Tally};
    use array_vis_bench_traits::DirectSort;
    use sort_logger::NoOpLogger;

    #[test]
    fn the_left_subtree_of_a_complete_tree_is_the_bigger_half_of_the_last_level() {
        assert_eq!((1..=8).map(left_size).collect::<Vec<_>>(), [0, 1, 1, 2, 3, 3, 3, 4]);
        for m in 2..2000 {
            let (left, right) = (left_size(m), m - 1 - left_size(m));
            let depth = |k: usize| usize::BITS - k.leading_zeros();
            assert!(left >= right && depth(left) <= depth(right) + 1, "m={m}: {left} and {right}");
        }
    }

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

    // The three invariants of the live heap `arr[..end]`: the min heap property;
    // untagged equals in original order along the cells; and no untagged equal
    // of a tagged cell anywhere in its subtree, which is what lets a right child
    // rise over a tagged left child.
    fn is_a_stable_min_heap(arr: &[Item], end: usize, tags: &Tags) -> bool {
        let mut stack = vec![(0usize, arr.len(), Vec::<u32>::new())];
        while let Some((p, m, tagged_above)) = stack.pop() {
            if !tags.contains_key(&p) && tagged_above.contains(&arr[p].key) { return false; }
            let mut above = tagged_above;
            if tags.contains_key(&p) { above.push(arr[p].key); }
            let left = left_size(m);
            for (c, cm) in [(p + 1, left), (p + 1 + left, m - 1 - left)] {
                if cm == 0 || c >= end { continue; }
                if arr[c].key < arr[p].key { return false; }
                stack.push((c, cm, above.clone()));
            }
        }
        let mut last = HashMap::new();
        (0..end).filter(|c| !tags.contains_key(c)).all(|c| {
            let x = arr[c];
            last.insert(x.key, x.tag).is_none_or(|t| t < x.tag)
        })
    }

    #[test]
    fn the_build_is_a_min_heap_with_equals_in_order() {
        let mut rng = Rng(0x8EAD);
        for n in [1usize, 2, 3, 4, 5, 7, 8, 9, 31, 32, 33, 100, 257, 600] {
            for kind in 0..KINDS {
                let mut arr = input(kind, n, &mut rng);
                build(&mut arr, 0, n, &mut NoOpLogger);
                assert!(is_a_stable_min_heap(&arr, n, &Tags::default()), "kind {kind}, n={n}: {:?}", pairs(&arr));
            }
        }
    }

    #[test]
    fn every_pop_leaves_a_stable_min_heap() {
        let mut rng = Rng(0x7EA);
        for n in [2usize, 3, 5, 8, 31, 64, 100, 257] {
            for kind in 0..KINDS {
                let mut arr = input(kind, n, &mut rng);
                run(&mut arr, &mut NoOpLogger, |arr, end, tags| {
                    assert!(is_a_stable_min_heap(arr, end, tags), "kind {kind}, n={n}, live {end}: {:?} tags {tags:?}", pairs(arr));
                });
            }
        }
    }

    // On random distinct data the map holds the jumpers still in the heap, which
    // peaks well under half the elements; the bound is the claim the space makes.
    #[test]
    fn the_tags_peak_under_half_the_elements_on_distinct_data() {
        let n = 1 << 14;
        let mut rng = Rng(0x7A6);
        let mut arr: Vec<u32> = (0..n).map(|_| rng.next(1 << 30) as u32).collect();
        let mut peak = 0;
        run(&mut arr, &mut NoOpLogger, |_, _, tags| peak = peak.max(tags.len()));
        assert!(peak < n / 2, "peak {peak} tags for {n} elements");
    }

    #[test]
    fn sorts_stably() {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 31, 32, 33, 64, 100, 257, 600, 1500, 3001] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                TaggedHeapSort::sort(&mut arr, &mut NoOpLogger);
                let mut want = arr0.clone();
                want.sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "kind {kind}, n={n}");
            }
        }
    }

    #[test]
    fn the_direct_sort_role_sorts() {
        let mut arr: Vec<usize> = (0..100).rev().collect();
        <TaggedHeapSort as DirectSort>::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn no_move_goes_unlogged() {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                TaggedHeapSort::sort(&mut arr, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "a move went unlogged, n={n} values={values}");
            }
        }
    }

    // The build is `n log n` comparisons and up to as many moves, the pops
    // about twice that in comparisons, so four times `n log n` bounds both.
    #[test]
    fn costs_n_log_n_comparisons_and_moves() {
        let n = 4096;
        let mut rng = Rng(0xC0FFEE);
        let mut arr: Vec<usize> = (0..n).map(|_| rng.next(1 << 30)).collect();
        let mut tally = Tally::default();
        TaggedHeapSort::sort(&mut arr, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 4 * n * 12, "comparisons {}", tally.cmps);
        assert!(tally.moves < 4 * n * 12, "moves {}", tally.moves);
    }
}
