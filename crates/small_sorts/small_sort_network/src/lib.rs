//! Optimal 8-element sorting network (19 comparators, 6 stages).
//! Falls back to linear insertion for smaller sizes.

use array_vis_bench_traits::{
    insertion_sort_with, Complexity, HasSpace, HasStability, HasTimeBounds, NonTrivialSmallSort,
    SmallSort,
};
use small_sort_insertion_strategy::LinearInsertion;
use sort_logger::SortLogger;

pub struct NetworkSmallSort;

impl SmallSort for NetworkSmallSort {
    const THRESHOLD: usize = 8;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        if arr.len() == 8 {
            sort_network_8(arr, logger)
        } else {
            insertion_sort_with::<LinearInsertion, _, _>(arr, logger)
        }
    }
}
impl NonTrivialSmallSort for NetworkSmallSort {}

impl HasTimeBounds for NetworkSmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for NetworkSmallSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for NetworkSmallSort {
    // Batcher-style network — not stable.
    const STABLE: bool = false;
}

/// Optimal 8-element sorting network (19 comparators, 6 stages).
/// Returns `true` if the array was mutated. `pub` so siblings (e.g.
/// `small_sort_network_16`) can chain to it instead of duplicating the
/// fast path.
#[inline(always)]
pub fn sort_network_8<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut mutated = false;
    // Stage 1
    mutated |= logger.cond_swap_gt(arr, 0, 1);
    mutated |= logger.cond_swap_gt(arr, 2, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 7);
    // Stage 2
    mutated |= logger.cond_swap_gt(arr, 0, 2);
    mutated |= logger.cond_swap_gt(arr, 1, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 6);
    mutated |= logger.cond_swap_gt(arr, 5, 7);
    // Stage 3
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    // Stage 4
    mutated |= logger.cond_swap_gt(arr, 0, 4);
    mutated |= logger.cond_swap_gt(arr, 1, 5);
    mutated |= logger.cond_swap_gt(arr, 2, 6);
    mutated |= logger.cond_swap_gt(arr, 3, 7);
    // Stage 5
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    // Stage 6
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated
}
