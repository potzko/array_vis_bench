//! `RotationMerge`, the merge role: merge two adjacent sorted runs in place.
//! `run_rotation_merge` is the one-shot that owns the scratch buffer, the merge
//! analogue of `rotation::run_rotation`. `MergeStrategy` is a rotation merge
//! left open in its rotation. The fixed size arrays a merge keeps, which a C
//! version holds on the stack, are the words its caller allocates once.

use crate::role::with_words;
use crate::Rotation;
use sort_logger::SortLogger;

/// Strategy for merging two adjacent sorted slices in-place using rotation.
///
/// `scratch_size(n)` returns the size of an aux buffer the caller should
/// pre-allocate; `merge` then receives that buffer and forwards it to
/// every `R::rotate` invocation so the visualiser shows a single aux
/// array per sort run rather than one per rotation.
pub trait RotationMerge {
    /// Required scratch size for merging an array of length `n`. Delegates
    /// to the underlying rotation by default.
    fn scratch_size(n: usize) -> usize;

    /// Lengths of the fixed size arrays of `usize` words the merge keeps, the
    /// ones a C version holds on the stack: bit masks, counts. The caller
    /// allocates them once, as one array in sections drawn side by side, by
    /// `with_words`, and hands the whole to every call as `words`, so the
    /// visualiser shows them and every write to them is logged. A merge without
    /// any leaves the default and gets an empty slice.
    const SCRATCH_SECTIONS: &'static [usize] = &[];

    /// Merge `arr[..mid]` and `arr[mid..]` in-place, using `scratch` as
    /// caller-owned aux memory passed down to the rotation, and `words`, the
    /// sections of `SCRATCH_SECTIONS` laid end to end, holding what the calls
    /// before left.
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        words: &mut [usize],
        logger: &mut U,
    );
}

/// A rotation merge left open in its rotation, which the caller names. A block
/// merge sort merges with one where its buffer and its keys do not reach, by the
/// rotation it rotates with everywhere else, so the merge is chosen apart from
/// the rotation.
pub trait MergeStrategy {
    type With<R: Rotation>: RotationMerge;
}

/// Run a single [`RotationMerge`] as a one-shot: allocate the merge's scratch
/// buffer and words, merge `arr[..mid]` with `arr[mid..]`, then free them. The
/// standalone `Category::Merge` entries use this. A repeated caller (a merge
/// sort) allocates them once at its top level instead.
pub fn run_rotation_merge<M, T, U>(arr: &mut [T], mid: usize, logger: &mut U)
where
    M: RotationMerge,
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    let scratch_size = M::scratch_size(arr.len());
    with_words::<T, U, _>(logger, M::SCRATCH_SECTIONS, |logger, words| {
        if scratch_size == 0 {
            M::merge(arr, mid, &mut [], words, logger);
        } else {
            let mut scratch = logger.create_aux_arr_t(scratch_size);
            M::merge(arr, mid, &mut scratch, words, logger);
            logger.free_aux_arr_t(&scratch);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::marker::PhantomData;
    use sort_logger::{split_sections, SortLog, VisualizerLogger};

    // A merge by a full sort that keeps words of its own, two sections whose length
    // it checks, and writes into the second through the logger.
    struct Wordy;
    impl RotationMerge for Wordy {
        const SCRATCH_SECTIONS: &'static [usize] = &[2, 3];

        fn scratch_size(_n: usize) -> usize { 0 }

        fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _mid: usize, _scratch: &mut [T], words: &mut [usize], logger: &mut U) {
            assert_eq!(words.len(), 5, "the words of the merge");
            let [_, tail] = split_sections(&mut *words, [2, 3]);
            logger.write_data_u(tail, 2, arr.len());
            arr.sort();
        }
    }

    // A merge by a full sort that keeps none.
    struct Bare;
    impl RotationMerge for Bare {
        fn scratch_size(_n: usize) -> usize { 0 }

        fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _mid: usize, _scratch: &mut [T], words: &mut [usize], _logger: &mut U) {
            assert!(words.is_empty());
            arr.sort();
        }
    }

    fn run<M: RotationMerge>() -> Vec<SortLog<usize>> {
        let mut arr = vec![4, 5, 9, 1, 2, 3];
        let mut log = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        run_rotation_merge::<M, usize, _>(&mut arr, 3, &mut log);
        assert_eq!(arr, [1, 2, 3, 4, 5, 9]);
        log.log
    }

    // The one-shot allocates the words of the merge, an array a section, and
    // frees them after it, and a merge without words has no events of them.
    #[test]
    fn the_one_shot_allocates_and_frees_the_words_of_the_merge() {
        let events = run::<Wordy>();
        let [
            SortLog::CreateAuxArr { name: head, length: 2 },
            SortLog::CreateAuxArr { name: tail, length: 3 },
            SortLog::Beside { name: beside, of },
            SortLog::WriteDataU { name: written, ind: 2, data: 6 },
            SortLog::FreeAuxArr { name: freed_head },
            SortLog::FreeAuxArr { name: freed_tail },
        ] = events.as_slice()
        else {
            panic!("{events:?}")
        };
        assert_eq!((beside, of, written, freed_head, freed_tail), (tail, head, tail, head, tail));
        assert!(run::<Bare>().is_empty());
    }
}
