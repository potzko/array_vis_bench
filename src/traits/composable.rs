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
pub trait HasTimeBounds {
    /// Big-O: upper bound.
    const WORST: Complexity;
    /// Big-Omega: lower bound.
    const BEST: Complexity;
    /// Big-Theta when WORST == BEST; otherwise the expected case.
    const AVERAGE: Complexity;
}

/// Auxiliary-space complexity (heap allocations that grow with N).
/// Recursion stack depth is part of the *outer* sort's composition,
/// not of each axis component. Bounded stack buffers (e.g. trinity
/// rotation's set buffer) count as `O(1)`.
pub trait HasSpace {
    const SPACE: Complexity;
}

/// Whether the algorithm preserves the relative order of equal keys.
/// Composed via boolean AND across axes.
pub trait HasStability {
    const STABLE: bool;
}

/// Pivot-selection quality, used by quicksort-family worst-case
/// composition. `DEGENERATES = true` means the pivot can produce
/// pathologically unbalanced partitions on some inputs (e.g.
/// `FirstElement` on already-sorted input → O(N) recursion depth →
/// O(N²) total). `DEGENERATES = false` means partitions are guaranteed
/// balanced (e.g. median-of-medians → O(log N) depth).
pub trait PivotQuality {
    const DEGENERATES: bool;
}
