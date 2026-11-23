use std::marker::PhantomData;
use sort_logger::SortLogger;
use super::rotation_merge::RotationMerge;
use array_vis_bench_traits::role::with_words;
use array_vis_bench_traits::SmallSort;
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};

// Outer merge sort variants. Per-merge cost dominates the total — using
// NaiveRotationMerge (O(k²) per size-k merge) gives O(N²) overall;
// SmallerSideRotationMerge (O(k log² k) per merge) gives ≈ O(N log³ N)
// rounded to its dominant "N log² N" bucket for picker display.
/// Top-down (recursive) rotation merge sort.
///
/// Merges in-place using the `M` rotation strategy — no auxiliary array.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy (`NaiveRotation` or `SmallerSideRotation`).
/// - `EARLY_EXIT`: skip the merge when the two halves are already in order.
pub struct TopDownRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let scratch_size = M::scratch_size(n);
        with_words::<T, U, _>(logger, M::SCRATCH_SECTIONS, |logger, words| {
            if scratch_size == 0 {
                Self::sort_rec(arr, &mut [], words, logger);
            } else {
                let mut scratch = logger.create_aux_arr_t(scratch_size);
                Self::sort_rec(arr, &mut scratch, words, logger);
                logger.free_aux_arr_t(&scratch);
            }
        });
    }

    fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut [T],
        words: &mut [usize],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        if S::THRESHOLD > 0 && n <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mid = n / 2;
        {
            let (l, r) = arr.split_at_mut(mid);
            Self::sort_rec(l, scratch, words, logger);
            Self::sort_rec(r, scratch, words, logger);
        }
        if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
            return;
        }
        M::merge(arr, mid, scratch, words, logger);
    }
}

array_vis_bench_traits::direct_sort!(TopDownRotationMergeSort<S, M, EARLY_EXIT>; S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool);

// The per-merge rotation strategy dominates the whole-sort cost: M's declared
// WORST/AVERAGE (N_SQUARED for naive, N_LOG_SQUARED for smaller-side) already
// equals the legacy whole-sort big_o, so forward M's bounds directly. BEST is
// M's BEST (N1 — already-merged input walks the larger side once).
impl<S: SmallSort, M: RotationMerge + HasTimeBounds, const EARLY_EXIT: bool> HasTimeBounds
    for TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    const WORST: Complexity = <M as HasTimeBounds>::WORST;
    const BEST: Complexity = <M as HasTimeBounds>::BEST;
    const AVERAGE: Complexity = <M as HasTimeBounds>::AVERAGE;
}
// Rotation merges in-place; the only aux is whatever the rotation needs (CONST
// for reversal etc.), surfaced via M::SPACE.
impl<S: SmallSort, M: RotationMerge + HasSpace, const EARLY_EXIT: bool> HasSpace
    for TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    const SPACE: Complexity = <M as HasSpace>::SPACE;
}
// Stable only when both the merge and the small sort keep equal keys in order.
impl<S: SmallSort + HasStability, M: RotationMerge + HasStability, const EARLY_EXIT: bool>
    HasStability for TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    const STABLE: bool = <M as HasStability>::STABLE && S::STABLE;
}

/// Bottom-up (iterative) rotation merge sort.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy.
/// - `EARLY_EXIT`: skip the merge when a segment pair is already sorted.
pub struct BottomUpRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let scratch_size = M::scratch_size(n);
        with_words::<T, U, _>(logger, M::SCRATCH_SECTIONS, |logger, words| {
            if scratch_size == 0 {
                Self::sort_inner(arr, &mut [], words, logger);
            } else {
                let mut scratch = logger.create_aux_arr_t(scratch_size);
                Self::sort_inner(arr, &mut scratch, words, logger);
                logger.free_aux_arr_t(&scratch);
            }
        });
    }

    fn sort_inner<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut [T],
        words: &mut [usize],
        logger: &mut U,
    ) {
        let n = arr.len();
        let gap0 = if S::THRESHOLD > 0 { S::THRESHOLD } else { 1 };

        if S::THRESHOLD > 0 {
            let mut i = 0;
            while i < n {
                let end = (i + S::THRESHOLD).min(n);
                S::sort(&mut arr[i..end], logger);
                i += S::THRESHOLD;
            }
        }

        let mut gap = gap0;
        while gap < n {
            let mut i = 0;
            while i < n {
                let mid = (i + gap).min(n);
                let end = (i + 2 * gap).min(n);
                if mid < end {
                    if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
                        // already sorted — skip
                    } else {
                        M::merge(&mut arr[i..end], mid - i, scratch, words, logger);
                    }
                }
                i += 2 * gap;
            }
            gap *= 2;
        }
    }
}

array_vis_bench_traits::direct_sort!(BottomUpRotationMergeSort<S, M, EARLY_EXIT>; S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool);

// Same forwarding shape as the top-down rotation head: the per-merge rotation
// strategy M dictates the whole-sort bounds and space, and its stability joins
// the small sort's.
impl<S: SmallSort, M: RotationMerge + HasTimeBounds, const EARLY_EXIT: bool> HasTimeBounds
    for BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    const WORST: Complexity = <M as HasTimeBounds>::WORST;
    const BEST: Complexity = <M as HasTimeBounds>::BEST;
    const AVERAGE: Complexity = <M as HasTimeBounds>::AVERAGE;
}
impl<S: SmallSort, M: RotationMerge + HasSpace, const EARLY_EXIT: bool> HasSpace
    for BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    const SPACE: Complexity = <M as HasSpace>::SPACE;
}
// Stable only when both the merge and the small sort keep equal keys in order.
impl<S: SmallSort + HasStability, M: RotationMerge + HasStability, const EARLY_EXIT: bool>
    HasStability for BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    const STABLE: bool = <M as HasStability>::STABLE && S::STABLE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask_merge::MaskRotationMerge;
    use crate::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};
    use array_vis_bench_traits::DirectSort;
    use core::marker::PhantomData;
    use rotation_reversal::ReversalRotation;
    use sort_logger::{split_sections, SortLog, VisualizerLogger};

    // No small sort, so that the sort is merges all the way down.
    struct Unit;
    impl SmallSort for Unit {
        const THRESHOLD: usize = 0;
        fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool { false }
    }

    // A merge by the smaller side that keeps words of its own, two sections whose
    // length it checks and the second of which it writes to through the logger.
    struct Wordy;
    impl RotationMerge for Wordy {
        const SCRATCH_SECTIONS: &'static [usize] = &[2, 3];

        fn scratch_size(n: usize) -> usize { SmallerSideRotationMerge::<ReversalRotation>::scratch_size(n) }

        fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, scratch: &mut [T], words: &mut [usize], logger: &mut U) {
            assert_eq!(words.len(), 5, "the words of a merge");
            let [_, tail] = split_sections(&mut *words, [2, 3]);
            logger.write_data_u(tail, 2, arr.len());
            SmallerSideRotationMerge::<ReversalRotation>::merge(arr, mid, scratch, words, logger);
        }
    }

    // The events a sort of 3000 elements by `S` leaves: the arrays of words
    // created, with their lengths, those freed, and the writes to them.
    fn word_events<S: DirectSort>() -> (Vec<(usize, usize)>, Vec<usize>, usize) {
        let mut arr: Vec<usize> = (0..3000).map(|i| i * 7919 % 3001).collect();
        let mut log = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        S::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        let created = log.log.iter().filter_map(|e| if let SortLog::CreateAuxArr { name, length } = e { Some((*name, *length)) } else { None });
        let freed = log.log.iter().filter_map(|e| if let SortLog::FreeAuxArr { name } = e { Some(*name) } else { None });
        (created.collect(), freed.collect(), log.log.iter().filter(|e| matches!(e, SortLog::WriteDataU { .. })).count())
    }

    // The words are one allocation of the sort, an array a section, however many
    // merges use them, and a merge without words leaves no array of them.
    fn words_are_created_once<S: DirectSort, B: DirectSort>() {
        let (created, freed, writes) = word_events::<S>();
        assert_eq!(created.iter().map(|(_, length)| *length).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(freed, created.iter().map(|(name, _)| *name).collect::<Vec<_>>());
        assert!(writes > 100, "{writes} writes to the words: a merge each");
        assert_eq!(word_events::<B>(), (vec![], vec![], 0));
    }

    // The mask merge keeps three arrays of the words of its cap, drawn side by
    // side: a sort over it makes them once and frees them once, and every merge
    // of a window writes to them.
    #[test]
    fn a_sort_over_the_mask_merge_allocates_its_three_arrays_once() {
        for (created, freed, writes) in [
            word_events::<TopDownRotationMergeSort<Unit, MaskRotationMerge<ReversalRotation>, false>>(),
            word_events::<BottomUpRotationMergeSort<Unit, MaskRotationMerge<ReversalRotation>, false>>(),
            word_events::<TopDownRotationMergeSort<Unit, MaskRotationMerge<ReversalRotation, 2, false>, true>>(),
        ] {
            let lengths = created.iter().map(|(_, length)| *length).collect::<Vec<_>>();
            assert!(lengths == [16, 16, 16] || lengths == [2, 2, 2], "{lengths:?}");
            assert_eq!(freed, created.iter().map(|(name, _)| *name).collect::<Vec<_>>());
            assert!(writes > 100, "{writes} writes to the words: a window of a merge each");
        }
    }

    #[test]
    fn the_top_down_sort_allocates_the_words_of_its_merge_once() {
        words_are_created_once::<TopDownRotationMergeSort<Unit, Wordy, false>, TopDownRotationMergeSort<Unit, SmallerSideRotationMerge<ReversalRotation>, false>>();
        words_are_created_once::<TopDownRotationMergeSort<Unit, Wordy, true>, TopDownRotationMergeSort<Unit, NaiveRotationMerge<ReversalRotation>, true>>();
    }

    #[test]
    fn the_bottom_up_sort_allocates_the_words_of_its_merge_once() {
        words_are_created_once::<BottomUpRotationMergeSort<Unit, Wordy, false>, BottomUpRotationMergeSort<Unit, SmallerSideRotationMerge<ReversalRotation>, false>>();
        words_are_created_once::<BottomUpRotationMergeSort<Unit, Wordy, true>, BottomUpRotationMergeSort<Unit, NaiveRotationMerge<ReversalRotation>, true>>();
    }
}
