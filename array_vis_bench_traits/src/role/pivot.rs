//! `PivotSelector` and `DualPivotSelector` — pivot-selection roles.
//!
//! Leaf crates (`pivot_first`, `pivot_ninther`, …) implement
//! `PivotSelector` and live in their own tiny crates. Dual-pivot impls
//! (`CombinedSelector`, `NintherDualPivot`) currently still live in
//! `array_vis_bench`; they're closely coupled to the dual-pivot
//! quick-sort family and may move out in a later batch.
//!
//! `median_index` and `min_max_index` are shared helpers — multiple
//! pivots reach for them, so they live here next to the traits.

use sort_logger::SortLogger;

pub trait PivotSelector {
    /// Display name — used by the standalone partition / quick-select
    /// registration macros to spell out per-leaf path segments
    /// (`partitions/lomuto/<name>`).
    const NAME: &'static str;
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], logger: &mut U) -> usize;
}

/// Selects two pivot indices from a slice in a single call.
///
/// The returned indices `(p1, p2)` need not be ordered — the sort
/// algorithm normalises them. They *should* differ wherever possible.
pub trait DualPivotSelector {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> (usize, usize);
}

/// Unified pivot-source trait — produces `N` pivot indices into the
/// caller's slice. `N` is an associated const so the partition arity
/// can be matched against [`PartitionScheme::N_PIVOTS`] at the type
/// level; both single-pivot ([`PivotSelector`]) and dual-pivot
/// ([`DualPivotSelector`]) backends route through this.
///
/// A blanket impl lifts every [`PivotSelector`] into a `PivotInput`
/// with `N = 1`, so existing `QuickSort<P, FirstElement, SS>`
/// instantiations keep working unchanged. Dual-pivot types
/// (`CombinedSelector`, `NintherDualPivot`) impl `PivotInput`
/// directly with `N = 2`; they do *not* impl `PivotSelector`, so no
/// trait-coherence conflict.
pub trait PivotInput {
    /// Number of pivot indices written into `out` per call.
    const N: usize;
    /// Write `Self::N` pivot indices into `out[..Self::N]`. The
    /// indices need not be ordered — the partition normalises them.
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize],
    );
}

impl<V: PivotSelector> PivotInput for V {
    const N: usize = 1;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize],
    ) {
        out[0] = V::select(arr, logger);
    }
}

// ── NoPivot (N = 0) ──────────────────────────────────────────────────────────

/// Zero-pivot [`PivotInput`] for partitions that decide their own split
/// without consulting pivot values (e.g. midpoint splitters like the
/// heap-extract partition). `pick` writes nothing; the partition receives
/// an empty `pivots` slice.
pub struct NoPivot;

impl PivotInput for NoPivot {
    const N: usize = 0;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &[T],
        _logger: &mut U,
        _out: &mut [usize],
    ) {
    }
}

// ── Composable annotations for NoPivot ───────────────────────────────────────
//
// Conceptually a no-op pivot picker: zero work, zero space, vacuously stable.
// `DEGENERATES = false` because there's nothing to degenerate — partitions
// paired with `NoPivot` (like `HeapExtract`) decide their own balanced split,
// so QuickSort's worst-case recursion depth stays O(log N).

impl crate::composable::HasTimeBounds for NoPivot {
    const WORST: crate::Complexity = crate::Complexity::CONST;
    const BEST: crate::Complexity = crate::Complexity::CONST;
    const AVERAGE: crate::Complexity = crate::Complexity::CONST;
}
impl crate::composable::HasSpace for NoPivot {
    const SPACE: crate::Complexity = crate::Complexity::CONST;
}
impl crate::composable::HasStability for NoPivot {
    const STABLE: bool = true;
}
impl crate::composable::PivotQuality for NoPivot {
    const DEGENERATES: bool = false;
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Return `(min_index, max_index)` among `arr[a]`, `arr[b]`, `arr[c]`
/// using 3 comparisons (optimal for both min and max of 3).
pub fn min_max_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> (usize, usize) {
    let (lo, hi) = if logger.cmp_ge(arr, a, b) { (b, a) } else { (a, b) };
    if logger.cmp_ge(arr, c, hi) {
        (lo, c)
    } else if logger.cmp_ge(arr, lo, c) {
        (c, hi)
    } else {
        (lo, hi)
    }
}

/// Return the index whose value is the median among `arr[a]`, `arr[b]`,
/// `arr[c]`.
pub fn median_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    let a_le_b = logger.cmp_le(arr, a, b);
    let b_le_c = logger.cmp_le(arr, b, c);

    if a_le_b {
        if b_le_c {
            b
        } else if logger.cmp_le(arr, a, c) {
            c
        } else {
            a
        }
    } else if b_le_c {
        if logger.cmp_le(arr, a, c) {
            a
        } else {
            c
        }
    } else {
        b
    }
}
