//! Type-level annotations for sort algorithms.
//!
//! These traits express the *role* a sort type plays in composite algorithms.
//! They carry a `sort` method so they can be used directly as type bounds
//! in algorithms that need a helper sort (e.g. a merge sort base case, a
//! quick sort small-partition handler).
//!
//! # `SmallListSort`
//!
//! A sort efficient on small arrays — typically O(N²) algorithms whose
//! constant factors are low enough to win for small N.
//!
//! ## When to use
//!
//! Any algorithm that recursively (or iteratively) reduces a problem to
//! smaller sub-arrays should switch to a `SmallListSort` once the sub-array
//! is small enough.  The exact threshold is algorithm-specific.
//!
//! ## Provided implementations
//!
//! | Type                | Sort                          | Best for              |
//! |---------------------|-------------------------------|-----------------------|
//! | [`InsertionSmall`]  | Insertion sort                | nearly-sorted small   |
//! | [`ShellOpt256Small`]| Shell sort (Optimized-256)    | unordered small       |

use crate::traits::log_traits::SortLogger;

/// A sort that is efficient on small arrays.
///
/// Implement this for sorts suitable as base-case helpers in hybrid
/// algorithms.  The `sort` method takes `dyn SortLogger<usize>` so it can
/// be called from any context regardless of the outer logger type.
pub trait SmallListSort {
    fn sort(arr: &mut [usize], logger: &mut dyn SortLogger<usize>);
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

/// Insertion sort as a small-list helper.
///
/// Simple, cache-friendly, and O(N) on nearly-sorted sub-arrays — the most
/// common choice for hybrid algorithm base cases.
pub struct InsertionSmall;

impl SmallListSort for InsertionSmall {
    fn sort(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
        crate::sorts::insertion_sorts::insertion_sort::sort_dyn(arr, logger);
    }
}

/// Shell sort (Optimized-256 gap sequence) as a small-list helper.
///
/// Handles unordered small arrays faster than insertion sort because its
/// larger initial gaps fix big inversions in a single pass.  Prefer this
/// when the sub-array is unordered rather than nearly sorted.
pub struct ShellOpt256Small;

impl SmallListSort for ShellOpt256Small {
    fn sort(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
        use crate::sorts::shell_sorts::{sequences::Optimized256, shell_sort::ShellSort};
        ShellSort::<Optimized256>::sort(arr, logger);
    }
}
