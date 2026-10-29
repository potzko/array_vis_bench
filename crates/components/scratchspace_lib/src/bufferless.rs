//! The bufferless merge, `bufferless_merge`: a stable in place merge of two
//! adjacent sorted runs that owns no storage. It is symMerge over the rotation
//! `R`, so a merge of `n` elements costs `O(n)` comparisons and `O(n log n)`
//! moves through the logger. The `NoBuffer` and `NoScratch` backings, and any
//! backing whose buffer is too small for a run, merge with it.

use array_vis_bench_traits::role::{Rotation, RotationMerge};
use merge_sort_lib::SmallerSideRotationMerge;
use sort_logger::SortLogger;

/// Merge the sorted runs `arr[lo..mid]` and `arr[mid..hi]` in place, stably
/// (an equal key of the left run stays first). `scratch` is the rotation's,
/// `R::scratch_size` cells or more.
pub fn bufferless_merge<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    if lo >= mid || mid >= hi { return; }
    <SmallerSideRotationMerge<R> as RotationMerge>::merge(&mut arr[lo..hi], mid - lo, scratch, logger);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use core::cmp::Ordering;
    use sort_logger::NoOpLogger;

    // Ordered by `key` alone, so `tag` tells equal keys apart.
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

    // Two sorted runs tagged in order, merged under `R`, against a stable sort.
    fn merges_stably<R: Rotation>() {
        let mut state: u64 = 0xB0FF;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize % m
        };
        for round in 0..300 {
            let (la, lb, values) = (next(30), next(30), 1 + next(8));
            let mut keys: Vec<u32> = (0..la + lb).map(|_| next(values) as u32).collect();
            keys[..la].sort();
            keys[la..].sort();
            let mut arr: Vec<Item> = keys.into_iter().zip(0..).map(|(key, tag)| Item { key, tag }).collect();
            let mut want = arr.clone();
            want.sort();
            let mut scratch = vec![Item { key: 0, tag: 0 }; R::scratch_size(arr.len())];
            bufferless_merge::<R, _, _>(&mut arr, 0, la, la + lb, &mut scratch, &mut NoOpLogger);
            let tags = |items: &[Item]| items.iter().map(|x| x.tag).collect::<Vec<_>>();
            assert_eq!(tags(&arr), tags(&want), "{}, round {round}: la={la} lb={lb}", R::NAME);
        }
    }

    #[test]
    fn merges_stably_under_reversal() { merges_stably::<ReversalRotation>(); }

    // Trinity copies short sides through its scratch, so a merge that dropped the
    // scratch would panic here.
    #[test]
    fn hands_the_rotation_its_scratch() { merges_stably::<TrinityRotation>(); }
}
