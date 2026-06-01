//! Sentinel + tiny bounded small sorts.

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, NonTrivialSmallSort, SmallSort,
};
use sort_logger::SortLogger;

/// No small-sort: recurse / iterate all the way down to subarrays of
/// size 1.
pub struct NoSmallSort;

impl SmallSort for NoSmallSort {
    const THRESHOLD: usize = 0;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool {
        unreachable!("NoSmallSort::sort should never be called (THRESHOLD = 0)")
    }
}

impl HasTimeBounds for NoSmallSort {
    // Never invoked (THRESHOLD = 0). Pick a value that won't pollute
    // composition; CONST keeps `Complexity::product(_, CONST) = _`.
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for NoSmallSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for NoSmallSort {
    const STABLE: bool = true;
}

// ─────────────────────────────────────────────────────────────────────

/// Trivial small-sort: arrays of length ≤ 1 are already sorted; do nothing.
pub struct Size1SmallSort;

impl SmallSort for Size1SmallSort {
    const THRESHOLD: usize = 1;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool {
        false
    }
}

impl HasTimeBounds for Size1SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Size1SmallSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Size1SmallSort {
    const STABLE: bool = true;
}

// ─────────────────────────────────────────────────────────────────────

/// Small-sort for arrays of length ≤ 2: single conditional swap when
/// `len == 2`.
pub struct Size2SmallSort;

impl SmallSort for Size2SmallSort {
    const THRESHOLD: usize = 2;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        if arr.len() == 2 {
            logger.cond_swap_gt(arr, 0, 1)
        } else {
            false
        }
    }
}
impl NonTrivialSmallSort for Size2SmallSort {}

impl HasTimeBounds for Size2SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Size2SmallSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Size2SmallSort {
    const STABLE: bool = true;
}
