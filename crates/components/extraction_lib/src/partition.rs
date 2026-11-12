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

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        let n = hi - lo;
        if n == 0 { return lo; }
        if n == 1 { return lo + usize::from(log.cmp_lt_data(arr, lo, pivot)); }
        if n <= scratch.len() { return buffered(arr, lo, hi, pivot, scratch, log); }
        let mid = lo + n / 2;
        let left = Self::partition::<R, T, U>(arr, lo, mid, pivot, scratch, log);
        let right = Self::partition::<R, T, U>(arr, mid, hi, pivot, scratch, log);
        rotate_region::<R, T, U>(arr, left, mid, right, scratch, log);
        left + (right - mid)
    }
}

// One pass from the right: a match goes out to the scratch and an element that
// stays shifts right by the matches passed, so the matches' slots open at the
// front and the scratch empties into them in order. `n + m` moves for `m` matches.
fn buffered<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    pivot: T,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let mut out = 0;
    for i in (lo..hi).rev() {
        if log.cmp_lt_data(arr, i, pivot) {
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
        // The block is arr[first..first + found], the scan is at i.
        let (mut first, mut found) = (lo, 0);
        for i in lo..hi {
            if !log.cmp_lt_data(arr, i, pivot) { continue; }
            rotate_region::<R, T, U>(arr, first, first + found, i, scratch, log);
            first = i - found;
            found += 1;
        }
        rotate_region::<R, T, U>(arr, lo, first, first + found, scratch, log);
        lo + found
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

    // The expected outcome, spelled out: the elements below the pivot in order,
    // then the rest in order.
    fn reference(arr: &[Item], pivot: u32) -> Vec<(u32, u32)> {
        let below = arr.iter().filter(|x| x.key < pivot);
        let rest = arr.iter().filter(|x| x.key >= pivot);
        below.chain(rest).map(|x| (x.key, x.tag)).collect()
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
            let mut log = Cmps::default();
            let split = P::partition::<R, _, _>(&mut arr, lo, n, item(pivot), &mut scratch, &mut log);
            let want = reference(&arr0[lo..], pivot);
            let below = want.iter().take_while(|p| p.0 < pivot).count();
            assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "{}: round {round} touched the prefix", P::NAME);
            assert_eq!(pairs(&arr[lo..]), want, "{}, {}: round {round}: n={n} values={values} pivot={pivot} lo={lo}", P::NAME, R::NAME);
            assert_eq!(split, lo + below, "{}: round {round}: the split is off", P::NAME);
            assert_eq!(log.0, n - lo, "{}: round {round}: more than one comparison per element", P::NAME);
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
