//! The `Keys` backings. `InArrayKeys` tags the blocks of a rolling merge with
//! distinct elements carved out of the array itself, and `NoKeys` is the shape
//! that never rolls a block and so tags nothing.

use array_vis_bench_traits::role::keys::sort_region;
use array_vis_bench_traits::role::Keys;
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

/// The key region is part of the array: `blocks` distinct elements, sorted. A
/// merge tags its blocks by swapping a key into each, so the space is `O(1)` and
/// a rolled block still says which block it was.
pub struct InArrayKeys;
impl Keys for InArrayKeys {
    const NAME: &'static str = "in array";
    const MERGE_WORST: Complexity = Complexity::N1;
    const MERGE_SPACE: Complexity = Complexity::CONST;

    fn key_count(blocks: usize) -> usize { blocks }

    fn tag_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        key: usize,
        block: usize,
        log: &mut U,
    ) {
        log.swap(arr, key, block + 1);
    }

    fn find_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        first: usize,
        len: usize,
        count: usize,
        log: &mut U,
    ) -> usize {
        let tag = |slot: usize| first + slot * len + 1;
        (1..count).fold(0, |lowest, slot| if log.cmp_lt(arr, tag(slot), tag(lowest)) { slot } else { lowest })
    }

    fn restore<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        at: usize,
        count: usize,
        log: &mut U,
    ) {
        sort_region(arr, at, at + count, log);
    }
}

/// No key region: blocks never move, so the front of the window is always the
/// next block and nothing is tagged. A merge under this shape does not roll, and
/// a run past the buffer falls back to the rotation merge.
pub struct NoKeys;
impl Keys for NoKeys {
    const NAME: &'static str = "none";
    const MERGE_WORST: Complexity = Complexity::N_LOG_N;
    const MERGE_SPACE: Complexity = Complexity::CONST;

    fn key_count(_blocks: usize) -> usize { 0 }

    fn tag_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &mut [T],
        _key: usize,
        _block: usize,
        _log: &mut U,
    ) {
    }

    fn find_block<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &mut [T],
        _first: usize,
        _len: usize,
        _count: usize,
        _log: &mut U,
    ) -> usize {
        0
    }

    fn restore<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &mut [T],
        _at: usize,
        _count: usize,
        _log: &mut U,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sort_logger::NoOpLogger;

    #[test]
    fn a_tag_goes_into_the_second_slot_and_comes_out_again() {
        let mut arr = vec![10usize, 11, 12, 20, 21, 22, 1, 2];
        InArrayKeys::tag_block(&mut arr, 6, 3, &mut NoOpLogger);
        assert_eq!(arr, [10, 11, 12, 20, 1, 22, 21, 2]);
        InArrayKeys::tag_block(&mut arr, 6, 3, &mut NoOpLogger);
        assert_eq!(arr, [10, 11, 12, 20, 21, 22, 1, 2]);
    }

    #[test]
    fn the_lowest_tag_names_the_block() {
        // three blocks of three from index 1, tags in the second slot of each
        let arr_of = |tags: [usize; 3]| {
            vec![99usize, 50, tags[0], 51, 40, tags[1], 41, 30, tags[2], 31]
        };
        for (tags, slot) in [([30, 10, 20], 1), ([10, 20, 30], 0), ([20, 30, 10], 2)] {
            let mut arr = arr_of(tags);
            assert_eq!(InArrayKeys::find_block(&mut arr, 1, 3, 3, &mut NoOpLogger), slot);
        }
        let mut arr = arr_of([5, 6, 7]);
        assert_eq!(InArrayKeys::find_block(&mut arr, 1, 3, 1, &mut NoOpLogger), 0);
    }

    #[test]
    fn restore_sorts_the_key_region_and_nothing_else() {
        let mut arr = vec![9usize, 4, 2, 3, 1, 0, 9];
        InArrayKeys::restore(&mut arr, 1, 4, &mut NoOpLogger);
        assert_eq!(arr, [9, 1, 2, 3, 4, 0, 9]);
    }

    #[test]
    fn the_shape_without_keys_wants_none_and_does_nothing() {
        let mut arr = vec![3usize, 1, 2, 5, 4, 6];
        let before = arr.clone();
        assert_eq!(NoKeys::key_count(100), 0);
        NoKeys::tag_block(&mut arr, 0, 3, &mut NoOpLogger);
        assert_eq!(NoKeys::find_block(&mut arr, 0, 2, 3, &mut NoOpLogger), 0);
        NoKeys::restore(&mut arr, 0, 6, &mut NoOpLogger);
        assert_eq!(arr, before);
    }

    // The movement imitation end to end: tag the blocks, scramble them with
    // block swaps, then take them back in the original order by lowest tag and
    // release each tag, which must leave the key region as it was.
    #[test]
    fn tags_name_the_blocks_in_their_original_order_after_any_scramble() {
        let (blocks, len) = (7usize, 4usize);
        let first = blocks; // the key region comes first
        let mut state: u64 = 0x7A6;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize % m
        };
        for _ in 0..200 {
            // keys 0..blocks ascending, the blocks hold ids as 1000 * block + offset
            let mut arr: Vec<usize> = (0..blocks).collect();
            arr.extend((0..blocks).flat_map(|b| (0..len).map(move |o| 1000 * (b + 1) + o)));
            let initial = arr.clone();
            let n_keys = InArrayKeys::key_count(blocks);
            assert_eq!(n_keys, blocks);
            for b in 0..blocks { InArrayKeys::tag_block(&mut arr, b, first + b * len, &mut NoOpLogger); }
            for _ in 0..next(20) {
                let (a, b) = (next(blocks), next(blocks));
                if a != b { NoOpLogger.block_swap(&mut arr, first + a * len, first + b * len, len); }
            }
            for taken in 0..blocks {
                let window = first + taken * len;
                let slot = InArrayKeys::find_block(&mut arr, window, len, blocks - taken, &mut NoOpLogger);
                if slot != 0 { NoOpLogger.block_swap(&mut arr, window, window + slot * len, len); }
                assert_eq!(arr[window] / 1000, taken + 1, "block {taken} did not come out in order");
                InArrayKeys::tag_block(&mut arr, taken, window, &mut NoOpLogger);
            }
            assert_eq!(arr, initial, "keys and blocks are not whole again");
        }
    }
}
