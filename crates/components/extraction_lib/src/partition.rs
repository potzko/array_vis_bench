//! The `StablePartition` fillers. `RotationPartition` halves the region,
//! partitions each half and rotates the middle, in one buffered pass when the
//! scratch holds the region. `SlidingPartition` gathers the matches in one
//! block that rides behind the scan, as the scan extraction gathers its keys.

use array_vis_bench_traits::role::keys::rotate_region;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

/// Divide and conquer over the rotation: each half partitioned, then one
/// rotation brings the right half's matches up against the left half's. One
/// comparison per element and `O(n log n)` moves, `O(n log (n / s))` with `s`
/// cells of scratch, since a region the scratch holds is partitioned in one pass.
pub struct RotationPartition;
impl StablePartition for RotationPartition {
    const NAME: &'static str = "rotation";
    const MOVES: Complexity = Complexity::N_LOG_N;
    const MOVES_BUFFERED: Complexity = Complexity::N1;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_rotation::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_rotation::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

// `matches` says whether the element at an index is one of those to bring to
// the front, by one logged comparison against the pivot.
fn by_rotation<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let n = hi - lo;
    if n == 0 { return lo; }
    if n == 1 { return lo + usize::from(matches(log, arr, lo)); }
    if n <= scratch.len() { return buffered(arr, lo, hi, matches, scratch, log); }
    let mid = lo + n / 2;
    let left = by_rotation::<R, T, U, F>(arr, lo, mid, matches, scratch, log);
    let right = by_rotation::<R, T, U, F>(arr, mid, hi, matches, scratch, log);
    rotate_region::<R, T, U>(arr, left, mid, right, scratch, log);
    left + (right - mid)
}

// One pass from the right: a match goes out to the scratch and an element that
// stays shifts right by the matches passed, so the matches' slots open at the
// front and the scratch empties into them in order. `n + m` moves for `m` matches.
fn buffered<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let mut out = 0;
    for i in (lo..hi).rev() {
        if matches(log, arr, i) {
            log.write_accross(arr, i, scratch, out);
            out += 1;
        } else if out > 0 {
            log.write(arr, i + out, i);
        }
    }
    for k in 0..out { log.write_accross(scratch, out - 1 - k, arr, lo + k); }
    lo + out
}

/// The matches gather in one block that rides behind the scan: a run of
/// elements that stay costs one rotation of the block past it, so the moves are
/// `O(n + m r)` for `m` matches in `r` runs, linear when the matches are few or
/// clustered and quadratic at worst. One comparison per element.
pub struct SlidingPartition;
impl StablePartition for SlidingPartition {
    const NAME: &'static str = "sliding";
    const MOVES: Complexity = Complexity::N_SQUARED;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_sliding::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        by_sliding::<R, T, U, _>(arr, lo, hi, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

fn by_sliding<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    matches: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    // The block is arr[first..first + found], the scan is at i.
    let (mut first, mut found) = (lo, 0);
    for i in lo..hi {
        if !matches(log, arr, i) { continue; }
        rotate_region::<R, T, U>(arr, first, first + found, i, scratch, log);
        first = i - found;
        found += 1;
    }
    rotate_region::<R, T, U>(arr, lo, first, first + found, scratch, log);
    lo + found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{item, items, pairs, Item, Rng};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::{NoOpLogger, SortLog, SortLogger};

    // The expected outcome, spelled out: the elements the predicate picks in
    // order, then the rest in order; and for the three way split, below, equal
    // and above, each in order.
    fn reference(arr: &[Item], picked: impl Fn(u32) -> bool) -> Vec<(u32, u32)> {
        let first = arr.iter().filter(|x| picked(x.key));
        let rest = arr.iter().filter(|x| !picked(x.key));
        first.chain(rest).map(|x| (x.key, x.tag)).collect()
    }

    fn three_way(arr: &[Item], pivot: u32) -> Vec<(u32, u32)> {
        let below = arr.iter().filter(|x| x.key < pivot);
        let equal = arr.iter().filter(|x| x.key == pivot);
        let above = arr.iter().filter(|x| x.key > pivot);
        below.chain(equal).chain(above).map(|x| (x.key, x.tag)).collect()
    }

    // Counts comparisons, to pin one per element.
    #[derive(Default)]
    struct Cmps(usize);
    impl SortLogger<Item> for Cmps {
        fn log(&mut self, event: SortLog<Item>) {
            if matches!(event, SortLog::CmpData { .. } | SortLog::CmpInArr { .. }) { self.0 += 1 }
        }
    }

    // Every partition, under every rotation, with the scratch the rotation asks
    // for or the `cells` given, leaves the elements before `lo` alone and the
    // region as the reference says.
    fn matches_the_reference<P: StablePartition, R: Rotation>(cells: Option<usize>) {
        let mut rng = Rng(0xB1A5);
        for round in 0..400 {
            let n = 1 + rng.next(60);
            let values = 1 + rng.next(n + 3);
            let arr0 = items(&(0..n).map(|_| rng.next(values) as u32).collect::<Vec<_>>());
            let lo = rng.next(3).min(n - 1);
            let pivot = rng.next(values + 2) as u32;
            let mut arr = arr0.clone();
            let mut scratch = vec![item(0); cells.unwrap_or(R::scratch_size(n))];
            let below = arr0[lo..].iter().filter(|x| x.key < pivot).count();
            let not_above = arr0[lo..].iter().filter(|x| x.key <= pivot).count();
            let name = format!("{}, {}: round {round}: n={n} values={values} pivot={pivot} lo={lo}", P::NAME, R::NAME);
            let mut log = Cmps::default();
            let split = P::partition::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut log);
            assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{name}: touched the prefix");
            assert_eq!(pairs(&arr[lo..]), reference(&arr0[lo..], |k| k < pivot), "{name}: below");
            assert_eq!(split, lo + below, "{name}: the split is off");
            assert_eq!(log.0, n - lo, "{name}: more than one comparison per element");
            let mut arr = arr0.clone();
            let split = P::partition_at_most::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut NoOpLogger);
            assert_eq!(pairs(&arr[lo..]), reference(&arr0[lo..], |k| k <= pivot), "{name}: not above");
            assert_eq!(split, lo + not_above, "{name}: the at most split is off");
            let mut arr = arr0.clone();
            let (equal, above) = P::partition_three_way::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut NoOpLogger);
            assert_eq!(pairs(&arr[lo..]), three_way(&arr0[lo..], pivot), "{name}: three way");
            assert_eq!((equal, above), (lo + below, lo + not_above), "{name}: the three way splits are off");
        }
    }

    #[test]
    fn the_rotation_partition_matches_the_reference() {
        matches_the_reference::<RotationPartition, ReversalRotation>(None);
        matches_the_reference::<RotationPartition, TrinityRotation>(None);
        matches_the_reference::<RotationPartition, AuxiliaryRotation>(None);
    }

    // A scratch of any size: none, a few cells so the leaves are buffered and the
    // levels above them rotate, and the whole region in one pass.
    #[test]
    fn the_rotation_partition_takes_any_scratch() {
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(0));
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(5));
        matches_the_reference::<RotationPartition, ReversalRotation>(Some(64));
    }

    #[test]
    fn the_sliding_partition_matches_the_reference() {
        matches_the_reference::<SlidingPartition, ReversalRotation>(None);
        matches_the_reference::<SlidingPartition, TrinityRotation>(None);
        matches_the_reference::<SlidingPartition, AuxiliaryRotation>(None);
    }

    #[test]
    fn an_empty_region_and_an_unmoved_one_cost_nothing() {
        let mut arr = items(&[3, 1, 2]);
        let empty = RotationPartition::partition::<ReversalRotation, _, _>(&mut arr, 2, 2, item(9), &mut [], &mut NoOpLogger);
        assert_eq!(empty, 2);
        let all = SlidingPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, 3, item(9), &mut [], &mut NoOpLogger);
        assert_eq!((all, pairs(&arr)), (3, vec![(3, 0), (1, 1), (2, 2)]));
        let none = SlidingPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, 3, item(0), &mut [], &mut NoOpLogger);
        assert_eq!((none, pairs(&arr)), (0, vec![(3, 0), (1, 1), (2, 2)]));
    }
}
