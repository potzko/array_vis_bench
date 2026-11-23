//! `StablePartition`, the stable partition role: the elements of a region
//! below a pivot value brought to its front and the rest after them, each side
//! keeping its order. The partition extraction of a block merge sort selects
//! its sample with one, and a stable quick sort partitions with one. A filler
//! rotates with the `Rotation` it is handed and may use the scratch as a buffer
//! of its own, which the caller allocates once for the two together.

use crate::Complexity;
use crate::Rotation;
use sort_logger::SortLogger;

pub trait StablePartition {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Worst case moves of a partition of `n` elements without scratch.
    const MOVES: Complexity;

    /// Cells of scratch a partition of up to `n` elements wants for itself, 0
    /// for a filler that makes do with the rotation's. The caller allocates the
    /// larger of this and `R::scratch_size(n)` once and hands it to every
    /// call: the partition lets go of its cells before it rotates, so the two
    /// never use them at once.
    #[inline]
    fn scratch_size(_n: usize) -> usize { 0 }

    /// Move the elements of `arr[lo..hi]` below `pivot` to its front, each side
    /// keeping its order, and return where the rest begins. `scratch` holds
    /// `R::scratch_size(n)` or `Self::scratch_size(n)` cells, whichever is more,
    /// for an `n` no smaller than the region.
    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize;

    /// The same with the elements not above `pivot` first.
    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize;

    /// Below, equal to and above `pivot`, each part in order: where the equal
    /// part begins and where the above part begins. A selection needs the equal
    /// part to stop in, since a region of one value never shrinks otherwise.
    fn partition_three_way<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> (usize, usize) {
        let equal = Self::partition::<R, T, U>(arr, lo, hi, pivot, scratch, log);
        let above = Self::partition_at_most::<R, T, U>(arr, equal, hi, pivot, scratch, log);
        (equal, above)
    }
}
