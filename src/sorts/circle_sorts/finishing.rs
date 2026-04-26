//! Circle sort finishing strategies.
//!
//! Circle sort makes rapid early progress — after ⌊log₂(n)⌋ passes the
//! array is nearly sorted.  Rather than running to full convergence, a
//! *short circuit* switches to a sort that is O(N) on nearly-sorted data.
//!
//! # Two abstractions
//!
//! ## `NearSortedSort` — what to run after giving up
//!
//! A sort that is efficient when the input is already nearly sorted.
//! Implement this trait to define a new finishing sort for the short circuit.
//! For now only [`InsertionNearSort`] is provided.
//!
//! ## `FinishingStrategy` — when to stop and what to do
//!
//! Controls the outer driver loop:
//! - [`Convergence`] — run until no swap occurs (no short circuit).
//! - [`ShortCircuit<S>`] — stop after ⌊log₂(n)⌋ passes, then run `S`.

use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;

use super::{directions::BottomUpDirection, orderings::RecursiveOrder};

// ---------------------------------------------------------------------------
// NearSortedSort
// ---------------------------------------------------------------------------

/// A sort that is efficient when the input is already nearly sorted.
///
/// Used as the `S` in [`ShortCircuit<S>`].  Implement this trait for any sort
/// that runs fast (ideally O(N)) on nearly-sorted input.
pub trait NearSortedSort {
    /// Human-readable name used in tree menu paths (e.g. `"insertion"`).
    const NAME: &'static str;

    fn sort(arr: &mut [usize], logger: &mut dyn SortLogger<usize>);
}

/// Insertion sort as a nearly-sorted finishing sort.
///
/// Insertion sort runs in O(N) on nearly-sorted input, making it the
/// canonical choice for the circle-sort short circuit.
pub struct InsertionNearSort;

impl NearSortedSort for InsertionNearSort {
    const NAME: &'static str = "insertion";

    fn sort(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
        crate::utils::small_sort::insertion_sort(arr, logger);
    }
}

// ---------------------------------------------------------------------------
// FinishingStrategy
// ---------------------------------------------------------------------------

/// Controls when to stop the main circle-sort loop and what to do after.
pub trait FinishingStrategy {
    /// Maximum number of outer passes before handing off to [`Self::finish`].
    /// `None` = run to full convergence (no short circuit).
    fn max_passes(n: usize) -> Option<usize>;

    /// Called once the pass limit is reached (or convergence when no limit).
    fn finish(arr: &mut [usize], logger: &mut dyn SortLogger<usize>);
}

/// No short circuit — run the main algorithm until no swap occurs.
pub struct Convergence;

/// Short circuit: stop after ⌊log₂(n)⌋ passes, then run `S` to finish.
///
/// `S` must implement [`NearSortedSort`].  The `S` type is what you vary
/// to change the finishing sort; the pass limit is always ⌊log₂(n)⌋.
pub struct ShortCircuit<S>(PhantomData<S>);

impl FinishingStrategy for Convergence {
    fn max_passes(_n: usize) -> Option<usize> {
        None
    }
    fn finish(_arr: &mut [usize], _logger: &mut dyn SortLogger<usize>) {}
}

impl<S: NearSortedSort> FinishingStrategy for ShortCircuit<S> {
    fn max_passes(n: usize) -> Option<usize> {
        Some((n as f64).log2() as usize)
    }
    fn finish(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
        S::sort(arr, logger);
    }
}

// ---------------------------------------------------------------------------
// Driver functions
// ---------------------------------------------------------------------------

/// Run a bottom-up circle sort with the given finishing strategy.
///
/// Calls `Dir::run_iteration` until either `Finish::max_passes` is reached
/// or no swap occurs, then calls `Finish::finish`.
pub fn drive_bottom_up<Dir: BottomUpDirection, Finish: FinishingStrategy>(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
) {
    let max = Finish::max_passes(arr.len());
    let mut passes = 0;
    loop {
        if max.map_or(false, |m| passes >= m) {
            break;
        }
        if !Dir::run_iteration(arr, logger) {
            break;
        }
        passes += 1;
    }
    Finish::finish(arr, logger);
}

/// Run a recursive circle sort with the given finishing strategy.
pub fn drive_recursive<Order: RecursiveOrder, Finish: FinishingStrategy>(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
) {
    if arr.len() < 2 {
        return;
    }
    let max = Finish::max_passes(arr.len());
    let mut passes = 0;
    loop {
        if max.map_or(false, |m| passes >= m) {
            break;
        }
        if !Order::sort_range(arr, 0, arr.len() - 1, logger) {
            break;
        }
        passes += 1;
    }
    Finish::finish(arr, logger);
}
