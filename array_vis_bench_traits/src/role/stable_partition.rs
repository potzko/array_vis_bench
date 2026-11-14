//! `StablePartition`, the stable partition role: the elements of a region
//! below a pivot value brought to its front and the rest after them, each side
//! keeping its order. The partition extraction of a block merge sort selects
//! its sample with one, and a stable quick sort would partition with one. A
//! filler rotates with the `Rotation` it is handed and may use that rotation's
//! scratch as a buffer. A caller with a buffer of elements and marks of its own
//! lying idle lends them as a `Workspace`, which a filler may work through.

use crate::role::Marks;
use crate::Complexity;
use crate::Rotation;
use sort_logger::SortLogger;

/// What a caller lends a partition beside the rotation's scratch: a buffer of
/// cells and as many clear marks. A buffer that `keeps` its elements holds real
/// ones, so it is swapped with and gets them all back, in any order; one that
/// does not is written before it is read. The marks are clear again when the
/// partition returns. `carved` is the region the marks are carved from, empty
/// when they are allocated.
pub struct Workspace<'a, M: Marks, T> {
    pub buffer: &'a mut [T],
    pub keeps: bool,
    pub marks: &'a mut M::Store,
    pub carved: &'a mut [T],
}

pub trait StablePartition {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Worst case moves of a partition of `n` elements without scratch.
    const MOVES: Complexity;

    /// The same with scratch for a constant fraction of the region, `MOVES`
    /// for a filler the scratch does not help.
    const MOVES_BUFFERED: Complexity = Self::MOVES;

    /// The same with no scratch but a workspace lent of the square root of the
    /// array or more, `MOVES` for a filler that does not work through one.
    const MOVES_LENT: Complexity = Self::MOVES;

    /// Move the elements of `arr[lo..hi]` below `pivot` to its front, each side
    /// keeping its order, and return where the rest begins. `scratch` is the
    /// rotation's, `R::scratch_size` cells or more.
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

    /// `partition` with a workspace lent, which a filler may ignore.
    #[allow(clippy::too_many_arguments)]
    fn partition_lent<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        _work: &mut Workspace<M, T>,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        Self::partition::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }

    /// `partition_at_most` with a workspace lent.
    #[allow(clippy::too_many_arguments)]
    fn partition_at_most_lent<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        _work: &mut Workspace<M, T>,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        Self::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }

    /// `partition_three_way` with a workspace lent to both of its splits.
    #[allow(clippy::too_many_arguments)]
    fn partition_three_way_lent<M: Marks, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        work: &mut Workspace<M, T>,
        scratch: &mut [T],
        log: &mut U,
    ) -> (usize, usize) {
        let equal = Self::partition_lent::<M, R, T, U>(arr, lo, hi, pivot, work, scratch, log);
        let above = Self::partition_at_most_lent::<M, R, T, U>(arr, equal, hi, pivot, work, scratch, log);
        (equal, above)
    }
}
