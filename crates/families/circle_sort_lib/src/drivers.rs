//! Spec-system drivers for the circle-sort family.
//!
//! The legacy registration path drives circle sort through the free functions
//! `drive_recursive` / `drive_bottom_up` in `finishing.rs`, which are
//! `usize`-locked (their `FinishingStrategy` / `NearSortedSort` traits are
//! declared over `&mut [usize]`). The spec emit backend instead calls a
//! GENERIC inherent `sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>` on a unique
//! type-head, and reads `<Ty as HasTimeBounds>::WORST` etc.
//!
//! This module supplies:
//!   * a generic [`CircleFinish`] trait (the spec-path replacement for the
//!     usize-locked `FinishingStrategy`/`NearSortedSort`),
//!   * two driver type-heads [`CircleRecursiveOf`] / [`CircleBottomUpOf`] that
//!     pair a convergence axis (order / direction) with a finish, each with an
//!     inherent generic `sort`, and
//!   * composable annotations for those heads (and for the zero-axis
//!     `CircleSortShakerRecursive`).

use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::circle_sort_shaker_recursive::CircleSortShakerRecursive;
use crate::directions::BottomUpDirection;
use crate::orderings::RecursiveOrder;

// ---------------------------------------------------------------------------
// Generic finishing axis (spec-path replacement for the usize-locked
// `FinishingStrategy` / `NearSortedSort` in finishing.rs).
// ---------------------------------------------------------------------------

/// When to stop the main circle-sort loop and what to run afterwards — the
/// generic, type-driven analogue of `finishing::FinishingStrategy`.
pub trait CircleFinish {
    /// Human-readable name (e.g. `"converge"`).
    const NAME: &'static str;

    /// Maximum number of outer passes before handing off to [`Self::finish`].
    /// `None` = run to full convergence (no short circuit).
    fn max_passes(n: usize) -> Option<usize>;

    /// Called once the pass limit is reached (or convergence when no limit).
    fn finish<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}

/// Run the main algorithm until no swap occurs (no short circuit).
pub struct ConvergeFinish;

impl CircleFinish for ConvergeFinish {
    const NAME: &'static str = "converge";

    fn max_passes(_n: usize) -> Option<usize> {
        None
    }

    fn finish<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) {}
}

/// Short circuit: stop after ⌊log₂(n)⌋ passes, then finish with linear
/// insertion sort (O(N) on nearly-sorted data).
pub struct InsertionShortCircuit;

impl CircleFinish for InsertionShortCircuit {
    const NAME: &'static str = "insertion";

    fn max_passes(n: usize) -> Option<usize> {
        Some((n as f64).log2() as usize)
    }

    fn finish<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let _ = array_vis_bench_traits::insertion_sort_with::<
            small_sort_insertion_strategy::LinearInsertion,
            _,
            _,
        >(arr, logger);
    }
}

// ---------------------------------------------------------------------------
// Recursive driver: type-head per (Order, Finish).
// ---------------------------------------------------------------------------

/// Recursive circle sort driven by an operation [`RecursiveOrder`] and a
/// [`CircleFinish`] strategy — the spec-system DRIVER.
///
/// Unlike `CircleSortRecursive<Order>` (convergence-only, no finish axis), this
/// wrapper reproduces both the converge and short-circuit legacy variants.
pub struct CircleRecursiveOf<Order: RecursiveOrder, Finish: CircleFinish>(
    PhantomData<(Order, Finish)>,
);

impl<Order: RecursiveOrder, Finish: CircleFinish> CircleRecursiveOf<Order, Finish> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
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
}

// ---------------------------------------------------------------------------
// Bottom-up driver: type-head per (Dir, Finish).
// ---------------------------------------------------------------------------

/// Bottom-up circle sort driven by a size-traversal [`BottomUpDirection`] and a
/// [`CircleFinish`] strategy — the spec-system DRIVER.
pub struct CircleBottomUpOf<Dir: BottomUpDirection, Finish: CircleFinish>(
    PhantomData<(Dir, Finish)>,
);

impl<Dir: BottomUpDirection, Finish: CircleFinish> CircleBottomUpOf<Dir, Finish> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
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
}

// ---------------------------------------------------------------------------
// Composable annotations (spec compiler reads these). Circle sort is
// ~Θ(N log² N) average; the legacy registration baked N_LOG_SQUARED across
// worst/best/average with CONST space and not-stable — matched here for parity.
// Bottom-up additionally allocates O(N) level buffers (directions.rs), so its
// SPACE is the more honest N1.
// ---------------------------------------------------------------------------

impl<Order: RecursiveOrder, Finish: CircleFinish> HasTimeBounds for CircleRecursiveOf<Order, Finish> {
    const WORST: Complexity = Complexity::N_LOG_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_SQUARED;
    const AVERAGE: Complexity = Complexity::N_LOG_SQUARED;
}
impl<Order: RecursiveOrder, Finish: CircleFinish> HasSpace for CircleRecursiveOf<Order, Finish> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<Order: RecursiveOrder, Finish: CircleFinish> HasStability for CircleRecursiveOf<Order, Finish> {
    const STABLE: bool = false;
}

impl<Dir: BottomUpDirection, Finish: CircleFinish> HasTimeBounds for CircleBottomUpOf<Dir, Finish> {
    const WORST: Complexity = Complexity::N_LOG_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_SQUARED;
    const AVERAGE: Complexity = Complexity::N_LOG_SQUARED;
}
impl<Dir: BottomUpDirection, Finish: CircleFinish> HasSpace for CircleBottomUpOf<Dir, Finish> {
    // Bottom-up allocates O(N) level buffers (Vec<(usize, usize)>) per sweep.
    const SPACE: Complexity = Complexity::N1;
}
impl<Dir: BottomUpDirection, Finish: CircleFinish> HasStability for CircleBottomUpOf<Dir, Finish> {
    const STABLE: bool = false;
}

// Zero-axis recursive shaker: convergence-only, in-place, not stable.
impl HasTimeBounds for CircleSortShakerRecursive {
    const WORST: Complexity = Complexity::N_LOG_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_SQUARED;
    const AVERAGE: Complexity = Complexity::N_LOG_SQUARED;
}
impl HasSpace for CircleSortShakerRecursive {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for CircleSortShakerRecursive {
    const STABLE: bool = false;
}
