//! How a block merge sort carves its keys and buffer, the `Extraction`
//! fillers: `ScanExtraction` is the one scan of `extract_distinct`,
//! `PrefixSortExtraction` sorts a prefix first so that the scan sweeps it for
//! free, and `PartitionExtraction` selects the smallest values by a
//! `StablePartition` around a pivot read off that prefix before sorting them.
//! `rotation_merge_sort` is the stable sort by rotations they share, and
//! `partition` holds the `StablePartition` fillers.

use core::marker::PhantomData;

use array_vis_bench_traits::role::keys::{extract_distinct, rotate_region, sort_region};
use array_vis_bench_traits::role::{Extraction, Rotation, StablePartition};
use scratchspace_lib::bufferless_merge;
use sort_logger::SortLogger;

pub mod partition;

#[cfg(test)]
pub mod test_util;

pub use partition::{BlockPartition, RotationPartition, SlidingPartition};

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

// The selection aims at OVERSHOOT times `want` elements and accepts up to SLACK
// times that, moving the pivot along the sample up to ROUNDS times to get there.
const OVERSHOOT: (usize, usize) = (3, 2);
const SLACK: usize = 2;
const ROUNDS: usize = 3;

/// The keys taken from the smallest values. The prefix of `PREFIX` times `want`
/// is sorted and read as a sample of the region: its element at the target's
/// rank is the pivot, and `P` partitions the rest of the region by it, so the
/// elements below it gather behind the prefix. A selection, the prefix's and
/// the gathered elements below the pivot, outside the window, under `want` or
/// over `SLACK` times the target, moves the pivot along the sample and
/// partitions again, the remainder when too few and the gathered part when too
/// many, `ROUNDS` times at most. The prefix's elements at the pivot and above
/// then step over the gathered part, which is sorted and merged into the
/// prefix's part below, and the scan compacts the first occurrences, carrying on
/// into the rest if the selection ran short of distinct values. Stable for the
/// prefix sort's reason, and because a partition keeps each side's order. The
/// point is the end of the sort: keys below the rest slide back in without a
/// move.
pub struct PartitionExtraction<P>(PhantomData<P>);

impl<P: StablePartition> Extraction for PartitionExtraction<P> {
    const NAME: &'static str = "partition";

    fn extract<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        if want == 0 || lo >= hi { return 0; }
        let n = hi - lo;
        let sample = (want * PREFIX.0).div_ceil(PREFIX.1).min(n);
        rotation_merge_sort::<R, T, U>(arr, lo, lo + sample, scratch, log);
        let target = (want * OVERSHOOT.0).div_ceil(OVERSHOOT.1);
        if target * 2 > n { return extract_distinct::<R, T, U>(arr, lo, hi, want, scratch, log); }
        // The sample's elements below the pivot are arr[lo..split], the gathered
        // ones arr[lo + sample..end], and `region` is what the next round
        // partitions: the remainder past `end`, or the gathered part when cutting.
        let (mut rank, mut end, mut split) = ((sample * target / n).min(sample - 1), lo + sample, lo);
        let mut region = (lo + sample, hi);
        for _ in 0..ROUNDS {
            let pivot = arr[lo + rank];
            end = P::partition::<R, T, U>(arr, region.0, region.1, pivot, scratch, log);
            split = rank_of(arr, lo, lo + sample, pivot, log);
            let selection = (split - lo) + (end - lo - sample);
            if selection < want {
                // At most a doubling of the rank: a selection far short of the
                // target says the sample's low end is sparse, and the proportional
                // step would gather many times the target, which the sliding
                // partition pays for by its square.
                let next = ((rank + 1) * target / selection.max(1)).clamp(rank + 1, 2 * rank + 2).min(sample - 1);
                if next == rank { break; }
                rank = next;
                region = (end, hi);
            } else if selection > SLACK * target {
                if rank == 0 { break; }
                rank = (rank * target / selection).min(rank - 1);
                region = (lo + sample, end);
            } else {
                break;
            }
        }
        rotate_region::<R, T, U>(arr, split, lo + sample, end, scratch, log);
        let gathered = end - (lo + sample);
        rotation_merge_sort::<R, T, U>(arr, split, split + gathered, scratch, log);
        bufferless_merge::<R, T, U>(arr, lo, split, split + gathered, scratch, log);
        extract_distinct::<R, T, U>(arr, lo, hi, want, scratch, log)
    }
}

/// First index of `arr[a..b]`, sorted, whose element is not below `pivot`.
fn rank_of<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], a: usize, b: usize, pivot: T, log: &mut U) -> usize {
    let (mut l, mut r) = (a, b);
    while l < r {
        let m = l + (r - l) / 2;
        if log.cmp_lt_data(arr, m, pivot) { l = m + 1; } else { r = m; }
    }
    l
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
    fn the_partition_extractions_carve_first_occurrences() {
        carves_first_occurrences::<PartitionExtraction<RotationPartition>, ReversalRotation>();
        carves_first_occurrences::<PartitionExtraction<RotationPartition>, TrinityRotation>();
        carves_first_occurrences::<PartitionExtraction<RotationPartition>, AuxiliaryRotation>();
        carves_first_occurrences::<PartitionExtraction<SlidingPartition>, ReversalRotation>();
        carves_first_occurrences::<PartitionExtraction<SlidingPartition>, AuxiliaryRotation>();
    }

    // The partition extraction's point: on distinct data the keys are the
    // smallest values of the region, so nothing of the rest is below them.
    #[test]
    fn the_partition_extraction_takes_the_smallest_values() {
        let mut rng = Rng(0x5A11);
        for round in 0..20 {
            let (n, want) = (2048 + rng.next(4096), 64 + rng.next(192));
            let arr0 = random(&mut rng, n, 1 << 30);
            let mut arr = arr0.clone();
            let found = PartitionExtraction::<RotationPartition>::extract::<ReversalRotation, _, _>(&mut arr, 0, n, want, &mut [], &mut NoOpLogger);
            assert_eq!(found, want);
            let highest_key = arr[want - 1].key;
            let lowest_rest = arr[want..].iter().map(|x| x.key).min().unwrap();
            assert!(highest_key < lowest_rest, "round {round}: n={n} want={want}: the keys reach {highest_key} and the rest starts at {lowest_rest}");
        }
    }

    // A sample whose second element is far below the rest reads as a target
    // rank of one, and the first partition gathers next to nothing. Growing the
    // rank in proportion would then jump to the sample's top and gather the
    // whole array, which the sliding partition pays for by its square; a
    // doubling at most keeps the rounds cheap.
    #[test]
    fn a_sparse_low_end_of_the_sample_does_not_gather_the_array() {
        const N: usize = 1 << 16;
        const WANT: usize = 256;
        let mut rng = Rng(0x10E);
        let mut arr = random(&mut rng, N, N);
        arr[0].key = 0;
        arr[1].key = 1;
        let mut tally = Tally::default();
        let found = PartitionExtraction::<SlidingPartition>::extract::<ReversalRotation, _, _>(&mut arr, 0, N, WANT, &mut [], &mut tally);
        assert_eq!(found, WANT);
        assert!(tally.moves < 10 * N, "{} moves for {N} elements", tally.moves);
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
