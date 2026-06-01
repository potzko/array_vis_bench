//! Bottom-up circle sort traversal directions.
//!
//! # What is bottom-up circle sort?
//!
//! The bottom-up family avoids recursion by iterating over circles of all sizes
//! explicitly, processing each level of the recursive call tree in one sweep.
//! The outer driver loop repeats until no swap occurs.
//!
//! The circles at each level are determined by the same splitting formula used
//! in the recursive version (`mid = l + (r-l)/2`), ensuring that all necessary
//! adjacent pairs are covered even for non-power-of-two array sizes.
//!
//! # Abstraction: bottom-up direction
//!
//! [`BottomUpDirection`] abstracts over the level-traversal strategy.
//! Each variant's `run_iteration` performs one full sweep (or pair of sweeps)
//! and returns whether any swap occurred.  The outer driver loop calls
//! `run_iteration` until it returns `false`.
//!
//! | Variant         | Order within one iteration                         |
//! |-----------------|---------------------------------------------------|
//! | `Decreasing`    | large circles first (BFS top-down)                |
//! | `Increasing`    | small circles first (BFS bottom-up)               |
//! | `ShakerDecInc`  | one top-down sweep, then one bottom-up sweep      |
//! | `ShakerIncDec`  | one bottom-up sweep, then one top-down sweep      |

use sort_logger::SortLogger;
use super::orderings::circle_pass;

// ---------------------------------------------------------------------------
// BFS helpers
// ---------------------------------------------------------------------------

/// One full BFS top-down sweep: processes the full-array circle first, then
/// level-1 circles, level-2 circles, etc. (large circles → small circles).
///
/// Circles are placed according to the recursive splitting formula
/// `mid = l + (r-l)/2`, which is identical to the recursive circle sort.
/// This ensures correct coverage of all adjacent pairs for any array length.
fn bfs_top_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let n = arr.len();
    if n < 2 {
        return false;
    }
    let mut swapped = false;
    let mut current: Vec<(usize, usize)> = vec![(0, n - 1)];
    let mut next: Vec<(usize, usize)> = Vec::new();
    while !current.is_empty() {
        for &(l, r) in &current {
            if l < r {
                if circle_pass(arr, l, r, logger) {
                    swapped = true;
                }
                let mid = l + (r - l) / 2;
                next.push((l, mid));
                next.push((mid + 1, r));
            }
        }
        std::mem::swap(&mut current, &mut next);
        next.clear();
    }
    swapped
}

/// One full BFS bottom-up sweep: processes the smallest circles first, working
/// up to the full-array circle (small circles → large circles).
///
/// Builds the complete level structure from the recursive splitting, then
/// processes levels from deepest to shallowest.
fn bfs_bottom_up<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let n = arr.len();
    if n < 2 {
        return false;
    }
    // Build all BFS levels from the recursive splitting tree.
    let mut levels: Vec<Vec<(usize, usize)>> = vec![vec![(0, n - 1)]];
    loop {
        let current = levels.last().unwrap();
        let mut next = Vec::new();
        for &(l, r) in current {
            if l < r {
                let mid = l + (r - l) / 2;
                next.push((l, mid));
                next.push((mid + 1, r));
            }
        }
        if next.is_empty() {
            break;
        }
        levels.push(next);
    }
    // Process from deepest level to the root.
    let mut swapped = false;
    for level in levels.iter().rev() {
        for &(l, r) in level {
            if l < r && circle_pass(arr, l, r, logger) {
                swapped = true;
            }
        }
    }
    swapped
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// How the bottom-up circle sort traverses circle levels in one outer
/// iteration.
///
/// Implement `run_iteration` to define a new traversal pattern.  The outer
/// driver loop in [`CircleSortBottomUp`] calls it until it returns `false`.
///
/// [`CircleSortBottomUp`]: super::circle_sort_bottom_up::CircleSortBottomUp
pub trait BottomUpDirection {
    /// Human-readable name shown in the selection menu.
    const NAME: &'static str;

    /// Perform one full sweep (or pair of sweeps) across all circle levels.
    ///
    /// Returns `true` if any swap occurred.
    fn run_iteration<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) -> bool;
}

// ---------------------------------------------------------------------------
// Concrete directions
// ---------------------------------------------------------------------------

/// Large-to-small: process from the full-width circle down to size-2 circles.
///
/// BFS top-down traversal of the recursive circle tree.  Large circles
/// establish global order first; smaller circles refine local order.
pub struct Decreasing;

/// Small-to-large: process from size-2 circles up to the full-width circle.
///
/// BFS bottom-up traversal.  Local order is established first, then
/// progressively merged into global order.
pub struct Increasing;

/// Shaker dec→inc: one top-down sweep followed by one bottom-up sweep per
/// outer iteration.
///
/// Short-circuits: if the top-down sweep produces no swaps, the bottom-up
/// sweep is skipped (the array is already sorted).
pub struct ShakerDecInc;

/// Shaker inc→dec: one bottom-up sweep followed by one top-down sweep per
/// outer iteration.
///
/// Mirror image of [`ShakerDecInc`].
pub struct ShakerIncDec;

impl BottomUpDirection for Decreasing {
    const NAME: &'static str = "decreasing";

    fn run_iteration<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) -> bool {
        bfs_top_down(arr, logger)
    }
}

impl BottomUpDirection for Increasing {
    const NAME: &'static str = "increasing";

    fn run_iteration<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) -> bool {
        bfs_bottom_up(arr, logger)
    }
}

impl BottomUpDirection for ShakerDecInc {
    const NAME: &'static str = "shaker dec\u{2192}inc";

    fn run_iteration<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) -> bool {
        bfs_top_down(arr, logger) && bfs_bottom_up(arr, logger)
    }
}

impl BottomUpDirection for ShakerIncDec {
    const NAME: &'static str = "shaker inc\u{2192}dec";

    fn run_iteration<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) -> bool {
        bfs_bottom_up(arr, logger) && bfs_top_down(arr, logger)
    }
}
