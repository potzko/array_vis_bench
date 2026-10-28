//! `Keys`, the key region of a block merge, the movement imitation buffer of
//! GrailSort and WikiSort. A block merge rolls the blocks of one run through the
//! other, which scrambles them, and a block cannot tell which of its siblings it
//! was. The key region is a run of distinct elements taken out of the array and
//! kept sorted: its `i`th key tags the `i`th block, by swapping the key into the
//! block, so the lowest tag still in the window names the next block in the
//! original order. Releasing a tag swaps it back, and the region is whole again.
//!
//! The axis has two shapes. In array keys carve the region from the array's own
//! distinct elements, `O(1)` space. No keys is the shape that never rolls a
//! block, so it has nothing to tag. `extract_distinct` and `sort_region` are the
//! shared mechanics, public because an in array buffer is carved the same way.

use crate::reverse;
use crate::Complexity;
use sort_logger::SortLogger;

pub trait Keys {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Worst-case time of one merge under this shape: `O(n)` when blocks roll,
    /// more when a run past the buffer falls back to a rotation merge.
    const MERGE_WORST: Complexity;

    /// Extra space of the key region.
    const MERGE_SPACE: Complexity;

    /// Distinct elements wanted for a merge that rolls `blocks` blocks, `0` for
    /// a shape that never rolls.
    fn key_count(blocks: usize) -> usize;

    /// Gather up to `want` distinct elements of `arr[lo..hi]` at its front, sorted,
    /// the rest keeping their order, and return how many were found. The first
    /// occurrence of each value is the one taken, so an element that stays is
    /// never ahead of an equal one that left. An in array buffer is carved by the
    /// same call, which is why the default is here and not in a backing.
    fn extract<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        log: &mut U,
    ) -> usize {
        extract_distinct(arr, lo, hi, want, log)
    }

    /// Tag the block starting at `block` with the key at `key`: swap the key with
    /// the block's second element. Applied again it releases the tag.
    fn tag_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        key: usize,
        block: usize,
        log: &mut U,
    );

    /// The slot, of `count` blocks of `len` elements from `first`, whose block
    /// carries the lowest tag: the next block in the original order.
    fn find_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        first: usize,
        len: usize,
        count: usize,
        log: &mut U,
    ) -> usize;

    /// Put the key region `arr[at..at + count]` back in order.
    fn restore<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        at: usize,
        count: usize,
        log: &mut U,
    );
}

/// Left rotate `arr[lo..hi]` so that `arr[mid..hi]` comes first. Three
/// reversals of sub slices of one array, every swap logged.
fn rotate_region<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    log: &mut U,
) {
    if lo == mid || mid == hi { return; }
    reverse(&mut arr[lo..mid], log);
    reverse(&mut arr[mid..hi], log);
    reverse(&mut arr[lo..hi], log);
}

/// First index of `arr[a..b]` whose element is not below `arr[at]`.
fn first_not_below<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    a: usize,
    b: usize,
    at: usize,
    log: &mut U,
) -> usize {
    let (mut l, mut r) = (a, b);
    while l < r {
        let m = l + (r - l) / 2;
        if log.cmp_lt(arr, m, at) { l = m + 1; } else { r = m; }
    }
    l
}

/// Gather up to `want` distinct elements of `arr[lo..hi]` at `lo`, sorted, and
/// return how many were found. The elements left behind keep their order. The
/// found keys form one sorted block that slides along behind the scan, so each
/// new key costs a binary search and two rotations: `O(n + want^2)` moves.
pub fn extract_distinct<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    want: usize,
    log: &mut U,
) -> usize {
    if want == 0 || lo >= hi { return 0; }
    // The keys are arr[first..first + found], the scan is at i.
    let (mut first, mut found, mut i) = (lo, 1, lo + 1);
    while i < hi && found < want {
        let rank = first_not_below(arr, first, first + found, i, log);
        if rank < first + found && !log.cmp_lt(arr, i, rank) {
            i += 1;
            continue;
        }
        let moved = rank + (i - first - found);
        rotate_region(arr, first, first + found, i, log);
        first = i - found;
        rotate_region(arr, moved, i, i + 1, log);
        found += 1;
        i += 1;
    }
    rotate_region(arr, lo, first, first + found, log);
    found
}

/// Insertion sort `arr[lo..hi]`, stable, every move logged.
pub fn sort_region<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    log: &mut U,
) {
    for i in lo + 1..hi {
        let x = arr[i];
        let mut j = i;
        while j > lo && log.cmp_gt_data(arr, j - 1, x) { j -= 1; }
        if j < i { log.shift_insert(arr, i, j, x); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;
    use sort_logger::NoOpLogger;

    // Ordered by `key` alone, so `tag` shows where an element came from.
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

    fn items(keys: &[u32]) -> Vec<Item> {
        keys.iter().zip(0u32..).map(|(&key, tag)| Item { key, tag }).collect()
    }

    fn keys_of(arr: &[Item]) -> Vec<u32> { arr.iter().map(|x| x.key).collect() }

    // The expected outcome, spelled out: the first occurrence of each of the
    // first `want` distinct values, sorted, then everything else in order.
    fn reference(arr: &[Item], want: usize) -> Vec<(u32, u32)> {
        let mut taken: Vec<Item> = Vec::new();
        for x in arr {
            if taken.len() < want && !taken.iter().any(|t| t.key == x.key) { taken.push(*x); }
        }
        taken.sort_by_key(|x| x.key);
        let rest = arr.iter().filter(|x| !taken.iter().any(|t| t.tag == x.tag));
        taken.iter().chain(rest).map(|x| (x.key, x.tag)).collect()
    }

    #[test]
    fn distinct_values_are_gathered_sorted_at_the_front() {
        let mut arr = items(&[5, 3, 9, 1, 7, 2]);
        let found = extract_distinct(&mut arr, 0, 6, 3, &mut NoOpLogger);
        assert_eq!(found, 3);
        assert_eq!(keys_of(&arr), [3, 5, 9, 1, 7, 2]);
    }

    #[test]
    fn a_repeated_value_is_taken_once_at_its_first_occurrence() {
        let mut arr = items(&[2, 2, 1, 3, 1, 2, 4]);
        let found = extract_distinct(&mut arr, 0, 7, 3, &mut NoOpLogger);
        assert_eq!(found, 3);
        let got: Vec<(u32, u32)> = arr.iter().map(|x| (x.key, x.tag)).collect();
        assert_eq!(got, [(1, 2), (2, 0), (3, 3), (2, 1), (1, 4), (2, 5), (4, 6)]);
    }

    #[test]
    fn too_few_distinct_values_take_what_there_is() {
        let mut arr = items(&[7, 7, 7, 7]);
        assert_eq!(extract_distinct(&mut arr, 0, 4, 3, &mut NoOpLogger), 1);
        assert_eq!(arr.iter().map(|x| x.tag).collect::<Vec<_>>(), [0, 1, 2, 3]);
        assert_eq!(extract_distinct(&mut arr, 0, 4, 0, &mut NoOpLogger), 0);
        assert_eq!(extract_distinct(&mut arr, 2, 2, 3, &mut NoOpLogger), 0);
    }

    #[test]
    fn extraction_matches_the_reference_on_random_arrays() {
        let mut state: u64 = 0xA11CE;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize % m
        };
        for round in 0..400 {
            let n = 1 + next(60);
            let values = 1 + next(n + 3);
            let want = next(n + 2);
            let arr0 = items(&(0..n).map(|_| next(values) as u32).collect::<Vec<_>>());
            let lo = next(3).min(n - 1);
            let mut arr = arr0.clone();
            let found = extract_distinct(&mut arr, lo, n, want, &mut NoOpLogger);
            let want_ref = reference(&arr0[lo..], want);
            assert_eq!(arr[..lo].iter().map(|x| x.tag).collect::<Vec<_>>(), (0..lo as u32).collect::<Vec<_>>());
            let got: Vec<(u32, u32)> = arr[lo..].iter().map(|x| (x.key, x.tag)).collect();
            assert_eq!(got, want_ref, "round {round}: n={n} values={values} want={want} lo={lo}");
            let distinct = want_ref.iter().take(found).map(|p| p.0).collect::<std::collections::BTreeSet<_>>();
            assert_eq!(distinct.len(), found, "round {round}: the found keys repeat a value");
        }
    }

    #[test]
    fn sort_region_is_stable_and_stays_inside_its_range() {
        let mut arr = items(&[9, 3, 3, 1, 2, 3, 8, 0]);
        sort_region(&mut arr, 1, 7, &mut NoOpLogger);
        let got: Vec<(u32, u32)> = arr.iter().map(|x| (x.key, x.tag)).collect();
        assert_eq!(got, [(9, 0), (1, 3), (2, 4), (3, 1), (3, 2), (3, 5), (8, 6), (0, 7)]);
    }
}
