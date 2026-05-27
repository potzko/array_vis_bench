//! Composable per-axis annotations for sort algorithms.
//!
//! Each generic-parameter axis (partition, pivot selector, small sort,
//! gap sequence, branching strategy, …) implements one or more of these
//! traits to declare *its own* contribution. The outer sort
//! (`QuickSort`, `BeapSort`, …) composes its parameters' values via
//! `Complexity::product` / `Complexity::sum` and boolean conjunction.
//!
//! All annotations are `const` so the resulting top-level values are
//! compile-time constants suitable for `#[distributed_slice]` statics.

use super::complexity::Complexity;

/// Time-complexity bounds. Most components have `WORST = BEST = AVERAGE`
/// (e.g. Lomuto partition is `O(N)` regardless of input shape); the
/// distinction matters for the *outer* sort, where worst-case can
/// differ from best/average depending on pivot behaviour.
///
/// All three consts default to [`Complexity::UNKNOWN`] (via `WORST`).
/// A component is free to skip every const if its bounds haven't been
/// analysed — its `Complexity::UNKNOWN` will then bubble up through any
/// composition that sums/products it. Implementations that *know* their
/// worst bound but not the tighter best/average override only `WORST`;
/// `BEST` and `AVERAGE` cascade to the same value.
pub trait HasTimeBounds {
    /// Big-O: upper bound. Defaults to [`Complexity::UNKNOWN`] —
    /// override when the bound is known.
    const WORST: Complexity = Complexity::UNKNOWN;
    /// Big-Omega: lower bound. Defaults to [`Self::WORST`] so a
    /// component that knows only its worst bound doesn't have to repeat
    /// it.
    const BEST: Complexity = Self::WORST;
    /// Big-Theta when WORST == BEST; otherwise the expected case.
    /// Defaults to [`Self::WORST`].
    const AVERAGE: Complexity = Self::WORST;
}

/// Auxiliary-space complexity (heap allocations that grow with N).
/// Recursion stack depth is part of the *outer* sort's composition,
/// not of each axis component. Bounded stack buffers (e.g. trinity
/// rotation's set buffer) count as `O(1)`.
///
/// Defaults to [`Complexity::UNKNOWN`]; override when the bound is known.
pub trait HasSpace {
    const SPACE: Complexity = Complexity::UNKNOWN;
}

/// Whether the algorithm preserves the relative order of equal keys.
/// Composed via boolean AND across axes. Defaults to `false` — assume
/// non-stable until proven otherwise.
pub trait HasStability {
    const STABLE: bool = false;
}

/// Pivot-selection quality, used by quicksort-family worst-case
/// composition. `DEGENERATES = true` means the pivot can produce
/// pathologically unbalanced partitions on some inputs (e.g.
/// `FirstElement` on already-sorted input → O(N) recursion depth →
/// O(N²) total). `DEGENERATES = false` means partitions are guaranteed
/// balanced (e.g. median-of-medians → O(log N) depth). Defaults to
/// `true` — worst-case assumption when the selector hasn't been
/// analysed.
pub trait PivotQuality {
    const DEGENERATES: bool = true;
}
