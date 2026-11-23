//! `Extraction`, the carving role of a block merge sort: how `want` distinct
//! elements are found in a region and gathered, sorted, at its front, each the
//! first occurrence of its value, the rest keeping their order among equals.
//! The scan in `keys` is the plain filler; the others prepare the region so
//! that the scan has less to do.

use crate::Rotation;
use sort_logger::SortLogger;

pub trait Extraction {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Cells of scratch an extraction of a region of up to `n` elements wants
    /// for itself, 0 for a filler that makes do with the rotation's. The
    /// caller allocates the larger of this and `R::scratch_size(n)` once.
    #[inline]
    fn scratch_size(_n: usize) -> usize { 0 }

    /// Lengths of the fixed size arrays of `usize` words the extraction keeps,
    /// as `StablePartition::SCRATCH_SECTIONS` has them, which a partition
    /// extraction takes over. The caller allocates them once, by `with_words`.
    const SCRATCH_SECTIONS: &'static [usize] = &[];

    /// Gather up to `want` distinct elements of `arr[lo..hi]` at its front,
    /// sorted, and return how many were found. Each is the first occurrence of
    /// its value, so an element that stays is never ahead of an equal one that
    /// left, and the elements that stay keep their order among equals. `scratch`
    /// holds `R::scratch_size(n)` or `Self::scratch_size(n)` cells, whichever is
    /// more, for an `n` no smaller than the region, and `words` the sections of
    /// `SCRATCH_SECTIONS`.
    fn extract<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize;
}
