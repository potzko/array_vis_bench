//! Recursive circle sort orderings.
//!
//! # What is circle sort?
//!
//! Circle sort repeatedly "zeroes in" on sorted order through a *circular*
//! comparison pattern.  In one pass over a range `[start, end]`, it compares
//! the outermost pair (`arr[start]` vs `arr[end]`), then the next pair
//! inward, and so on, swapping whenever the right element is smaller.  The
//! full sort repeats passes until no swap occurs.
//!
//! The *recursive* family also splits the range at the midpoint and applies
//! the same process to each half, giving O(N log² N) average behaviour.
//!
//! # Abstraction: recursive ordering
//!
//! At each recursive level there are three operations:
//!
//! 1. `circle_pass` — the circular comparison of `[start, end]`
//! 2. `recurse_left` — recurse on `[start, mid]`
//! 3. `recurse_right` — recurse on `[mid+1, end]`
//!
//! Their *relative order* is the only thing that differs across variants.
//! [`RecursiveOrder`] abstracts over this ordering.  Implementing a new
//! variant means writing one `sort_range` method with the desired sequence.

use crate::traits::log_traits::SortLogger;

// ---------------------------------------------------------------------------
// Shared primitive
// ---------------------------------------------------------------------------

/// Performs one circular comparison pass over `arr[start..=end]`.
///
/// Simultaneously compares `arr[start]` vs `arr[end]`, then the next pair
/// inward, etc., swapping whenever the right element is smaller.  For
/// odd-length ranges the middle element is also compared with its right
/// neighbour.
///
/// Returns `true` if any swap was made.
pub fn circle_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    let mut swapped = false;
    let (mut s, mut e) = (start, end);
    while s < e {
        if logger.cond_swap_lt(arr, e, s) {
            swapped = true;
        }
        s += 1;
        e -= 1;
    }
    if s == e && logger.cond_swap_lt(arr, e + 1, s) {
        swapped = true;
    }
    swapped
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// The ordering of the three operations at each recursive level.
///
/// Implement this trait to define a new ordering variant.  `sort_range`
/// should call [`circle_pass`] and recurse on both halves (`Self::sort_range`)
/// in the desired sequence, returning `true` if any swap occurred anywhere.
pub trait RecursiveOrder {
    /// Human-readable name shown in the selection menu (e.g. `"pre-order"`).
    const NAME: &'static str;

    /// Sort `arr[start..=end]` with this ordering.
    ///
    /// Returns `true` if any swap occurred (in the pass or in either
    /// sub-range).  The outer driver loop calls this until it returns `false`.
    fn sort_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) -> bool;
}

// ---------------------------------------------------------------------------
// Concrete orderings
// ---------------------------------------------------------------------------

/// Pre-order: `circle_pass` → `recurse_left` → `recurse_right`.
///
/// The classic circle sort ordering.  Comparisons run from the outside in
/// before the sub-ranges are refined.
pub struct PreOrder;

/// Left-mid-right: `recurse_left` → `circle_pass` → `recurse_right`.
///
/// The left half is refined first; then the circle pass runs on the whole
/// range; then the right half is refined.  Named "stooge order" after its
/// structural resemblance to stooge sort.
pub struct LeftMidRight;

/// Right-mid-left: `recurse_right` → `circle_pass` → `recurse_left`.
///
/// Mirror image of [`LeftMidRight`]: right half first, then pass, then left.
pub struct RightMidLeft;

/// Post-order: `recurse_left` → `recurse_right` → `circle_pass`.
///
/// Both halves are fully refined before the circle pass.  The pass acts as a
/// cleanup / merge step on already-partially-sorted halves.
pub struct PostOrder;

impl RecursiveOrder for PreOrder {
    const NAME: &'static str = "pre-order";

    fn sort_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) -> bool {
        if start == end {
            return false;
        }
        let pass = circle_pass(arr, start, end, logger);
        let mid = start + (end - start) / 2;
        let left = Self::sort_range(arr, start, mid, logger);
        let right = Self::sort_range(arr, mid + 1, end, logger);
        pass || left || right
    }
}

impl RecursiveOrder for LeftMidRight {
    const NAME: &'static str = "left-mid-right";

    fn sort_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) -> bool {
        if start == end {
            return false;
        }
        let mid = start + (end - start) / 2;
        let left = Self::sort_range(arr, start, mid, logger);
        let pass = circle_pass(arr, start, end, logger);
        let right = Self::sort_range(arr, mid + 1, end, logger);
        pass || left || right
    }
}

impl RecursiveOrder for RightMidLeft {
    const NAME: &'static str = "right-mid-left";

    fn sort_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) -> bool {
        if start == end {
            return false;
        }
        let mid = start + (end - start) / 2;
        let right = Self::sort_range(arr, mid + 1, end, logger);
        let pass = circle_pass(arr, start, end, logger);
        let left = Self::sort_range(arr, start, mid, logger);
        pass || left || right
    }
}

impl RecursiveOrder for PostOrder {
    const NAME: &'static str = "post-order";

    fn sort_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) -> bool {
        if start == end {
            return false;
        }
        let mid = start + (end - start) / 2;
        let left = Self::sort_range(arr, start, mid, logger);
        let right = Self::sort_range(arr, mid + 1, end, logger);
        let pass = circle_pass(arr, start, end, logger);
        pass || left || right
    }
}
