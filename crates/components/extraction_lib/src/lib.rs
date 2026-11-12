//! How a block merge sort carves its keys and buffer, the `Extraction`
//! fillers: `ScanExtraction` is the one scan of `extract_distinct`, and
//! `PrefixSortExtraction` sorts a prefix first so that the scan sweeps it for
//! free. `rotation_merge_sort` is the stable sort by rotations they share, and
//! `partition` holds the `StablePartition` fillers.

use array_vis_bench_traits::role::keys::{extract_distinct, sort_region};
use array_vis_bench_traits::role::{Extraction, Rotation};
use scratchspace_lib::bufferless_merge;
use sort_logger::SortLogger;

pub mod partition;

#[cfg(test)]
pub mod test_util;

pub use partition::{RotationPartition, SlidingPartition};

// Runs of RUN elements are insertion sorted before the first merge.
const RUN: usize = 8;

/// Stable sort of `arr[lo..hi]` by rotation merges: insertion sorted runs of
/// `RUN`, then `bufferless_merge` at doubling widths. `O(n log n)` comparisons
/// and `O(n log^2 n)` moves.
pub fn rotation_merge_sort<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    for start in (lo..hi).step_by(RUN) { sort_region(arr, start, (start + RUN).min(hi), log); }
    let mut width = RUN;
    while lo + width < hi {
        for a in (lo..hi - width).step_by(2 * width) {
            bufferless_merge::<R, T, U>(arr, a, a + width, (a + 2 * width).min(hi), scratch, log);
        }
        width *= 2;
    }
}

/// One scan, the keys gathered in a block riding behind it: `O(n + c^2)` moves
/// for `c` keys, as GrailSort and WikiSort gather theirs.
pub struct ScanExtraction;
impl Extraction for ScanExtraction {
    const NAME: &'static str = "scan";

    fn extract<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        extract_distinct::<R, T, U>(arr, lo, hi, want, scratch, log)
    }
}

// The prefix is `want` and a tenth, so a few repeats among the first elements
// still leave the keys inside it.
const PREFIX: (usize, usize) = (11, 10);

/// The first `PREFIX` times `want` elements sorted, stably, and the scan run
/// over them: a sorted prefix costs the scan nothing but one rotation of the
/// keys block past each run of repeats, and the scan carries on into the rest
/// when the prefix ran short of distinct values. Stable because the prefix is
/// the front: the first of each run of equals is still the array's first
/// occurrence of that value.
pub struct PrefixSortExtraction;
impl Extraction for PrefixSortExtraction {
    const NAME: &'static str = "prefix sort";

    fn extract<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        if want == 0 || lo >= hi { return 0; }
        let prefix = (want * PREFIX.0).div_ceil(PREFIX.1).min(hi - lo);
        rotation_merge_sort::<R, T, U>(arr, lo, lo + prefix, scratch, log);
        extract_distinct::<R, T, U>(arr, lo, hi, want, scratch, log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{item, items, pairs, Item, Rng};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::{NoOpLogger, SortLog, SortLogger};
    use std::collections::BTreeMap;

    // Counts the logged moves and comparisons.
    #[derive(Default)]
    pub struct Tally {
        pub moves: usize,
        pub cmps: usize,
    }
    impl SortLogger<Item> for Tally {
        fn log(&mut self, event: SortLog<Item>) {
            match event {
                SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => self.moves += 1,
                SortLog::CmpInArr { .. } | SortLog::CmpData { .. } => self.cmps += 1,
                _ => {}
            }
        }
    }

    fn random(rng: &mut Rng, n: usize, values: usize) -> Vec<Item> {
        items(&(0..n).map(|_| rng.next(values) as u32).collect::<Vec<_>>())
    }

    // The lowest tag of each key: the first occurrence of each value.
    fn firsts(arr: &[Item]) -> BTreeMap<u32, u32> {
        arr.iter().rev().map(|x| (x.key, x.tag)).collect()
    }

    // What every extraction promises, whatever keys it picks: as many as asked
    // or as there are, each the first occurrence of its value, sorted at the
    // front, the rest keeping their order among equals, the multiset whole and
    // the elements before `lo` untouched.
    pub fn carves_first_occurrences<E: Extraction, R: Rotation>() {
        let mut rng = Rng(0x5E1EC7);
        for round in 0..600 {
            let n = 1 + rng.next(if round < 500 { 60 } else { 400 });
            let values = 1 + rng.next(n + 3);
            let want = rng.next(n + 2);
            let arr0 = random(&mut rng, n, values);
            let lo = rng.next(3).min(n - 1);
            let mut arr = arr0.clone();
            let mut scratch = vec![item(0); R::scratch_size(n)];
            let found = E::extract::<R, _, _>(&mut arr, lo, n, want, &mut scratch, &mut NoOpLogger);
            let first = firsts(&arr0[lo..]);
            let name = format!("{}, {}: round {round}: n={n} values={values} want={want} lo={lo}", E::NAME, R::NAME);
            assert_eq!(found, want.min(first.len()), "{name}: found");
            assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{name}: the prefix moved");
            let keys = &arr[lo..lo + found];
            assert!(keys.windows(2).all(|w| w[0].key < w[1].key), "{name}: the keys are not sorted and distinct");
            assert!(keys.iter().all(|k| first[&k.key] == k.tag), "{name}: a key is not the first occurrence");
            let mut seen: BTreeMap<u32, u32> = BTreeMap::new();
            for x in &arr[lo + found..] {
                assert!(seen.insert(x.key, x.tag).is_none_or(|t| t < x.tag), "{name}: equals of {} reordered", x.key);
            }
            let (mut got, mut want_all) = (pairs(&arr), pairs(&arr0));
            got.sort();
            want_all.sort();
            assert_eq!(got, want_all, "{name}: the multiset changed");
        }
    }

    #[test]
    fn the_scan_carves_first_occurrences() {
        carves_first_occurrences::<ScanExtraction, ReversalRotation>();
        carves_first_occurrences::<ScanExtraction, TrinityRotation>();
    }

    #[test]
    fn the_prefix_sort_carves_first_occurrences() {
        carves_first_occurrences::<PrefixSortExtraction, ReversalRotation>();
        carves_first_occurrences::<PrefixSortExtraction, TrinityRotation>();
        carves_first_occurrences::<PrefixSortExtraction, AuxiliaryRotation>();
    }

    #[test]
    fn the_rotation_merge_sort_is_stable() {
        let mut rng = Rng(77);
        for _ in 0..200 {
            let (n, values) = (rng.next(70), 1 + rng.next(9));
            let arr0 = random(&mut rng, n, values);
            let lo = rng.next(4).min(n);
            let mut arr = arr0.clone();
            rotation_merge_sort::<ReversalRotation, _, _>(&mut arr, lo, n, &mut [], &mut NoOpLogger);
            let mut want = arr0.clone();
            want[lo..].sort_by_key(|x| x.key);
            assert_eq!(pairs(&arr), pairs(&want));
        }
    }

    // The prefix sort's point: on distinct data the scan then moves nothing, so
    // the carving costs the prefix sort alone, well under the scan's `c^2`.
    #[test]
    fn the_prefix_sort_moves_less_than_the_scan_on_distinct_data() {
        const N: usize = 8192;
        const WANT: usize = 512;
        let cost = |extract: fn(&mut [Item], &mut Tally) -> usize| {
            let mut arr = items(&(0..N).map(|i| (i * 2654435761usize % 1000003) as u32).collect::<Vec<_>>());
            let mut tally = Tally::default();
            assert_eq!(extract(&mut arr, &mut tally), WANT);
            tally.moves
        };
        let scan = cost(|arr, log| ScanExtraction::extract::<ReversalRotation, _, _>(arr, 0, N, WANT, &mut [], log));
        let prefix = cost(|arr, log| PrefixSortExtraction::extract::<ReversalRotation, _, _>(arr, 0, N, WANT, &mut [], log));
        assert!(prefix * 3 < scan, "prefix sort {prefix} moves against the scan's {scan}");
    }
}
