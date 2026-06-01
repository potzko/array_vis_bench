//! Batcher's odd-even merge sort network for 16 elements (63
//! comparators, 10 stages). Chains to the 8-element network for length
//! 8 and falls back to linear insertion below that.

use array_vis_bench_traits::{
    insertion_sort_with, Complexity, HasSpace, HasStability, HasTimeBounds, NonTrivialSmallSort,
    SmallSort,
};
use small_sort_insertion_strategy::LinearInsertion;
use small_sort_network::sort_network_8;
use sort_logger::SortLogger;

pub struct Network16SmallSort;

impl SmallSort for Network16SmallSort {
    const THRESHOLD: usize = 16;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        match arr.len() {
            16 => sort_network_16(arr, logger),
            8 => sort_network_8(arr, logger),
            _ => insertion_sort_with::<LinearInsertion, _, _>(arr, logger),
        }
    }
}
impl NonTrivialSmallSort for Network16SmallSort {}

impl HasTimeBounds for Network16SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Network16SmallSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Network16SmallSort {
    const STABLE: bool = false;
}

/// Batcher's odd-even merge sort network for 16 elements (63
/// comparators, 10 stages). Returns `true` if the array was mutated.
#[inline(always)]
fn sort_network_16<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut mutated = false;
    // Stage 1: sort pairs
    mutated |= logger.cond_swap_gt(arr, 0, 1);
    mutated |= logger.cond_swap_gt(arr, 2, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 9);
    mutated |= logger.cond_swap_gt(arr, 10, 11);
    mutated |= logger.cond_swap_gt(arr, 12, 13);
    mutated |= logger.cond_swap_gt(arr, 14, 15);
    // Stage 2: merge pairs → sorted 4s (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 2);
    mutated |= logger.cond_swap_gt(arr, 1, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 6);
    mutated |= logger.cond_swap_gt(arr, 5, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 10);
    mutated |= logger.cond_swap_gt(arr, 9, 11);
    mutated |= logger.cond_swap_gt(arr, 12, 14);
    mutated |= logger.cond_swap_gt(arr, 13, 15);
    // Stage 3: merge pairs → sorted 4s (fixup)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    // Stage 4: merge sorted 4s → sorted 8s (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 4);
    mutated |= logger.cond_swap_gt(arr, 1, 5);
    mutated |= logger.cond_swap_gt(arr, 2, 6);
    mutated |= logger.cond_swap_gt(arr, 3, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 12);
    mutated |= logger.cond_swap_gt(arr, 9, 13);
    mutated |= logger.cond_swap_gt(arr, 10, 14);
    mutated |= logger.cond_swap_gt(arr, 11, 15);
    // Stage 5: merge sorted 4s → sorted 8s (odd step)
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    mutated |= logger.cond_swap_gt(arr, 10, 12);
    mutated |= logger.cond_swap_gt(arr, 11, 13);
    // Stage 6: merge sorted 4s → sorted 8s (fixup)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 11, 12);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    // Stage 7: merge sorted 8s → sorted 16 (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 8);
    mutated |= logger.cond_swap_gt(arr, 1, 9);
    mutated |= logger.cond_swap_gt(arr, 2, 10);
    mutated |= logger.cond_swap_gt(arr, 3, 11);
    mutated |= logger.cond_swap_gt(arr, 4, 12);
    mutated |= logger.cond_swap_gt(arr, 5, 13);
    mutated |= logger.cond_swap_gt(arr, 6, 14);
    mutated |= logger.cond_swap_gt(arr, 7, 15);
    // Stage 8: merge sorted 8s → sorted 16 (odd step)
    mutated |= logger.cond_swap_gt(arr, 4, 8);
    mutated |= logger.cond_swap_gt(arr, 5, 9);
    mutated |= logger.cond_swap_gt(arr, 6, 10);
    mutated |= logger.cond_swap_gt(arr, 7, 11);
    // Stage 9: merge sorted 8s → sorted 16 (fixup 1)
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 8);
    mutated |= logger.cond_swap_gt(arr, 7, 9);
    mutated |= logger.cond_swap_gt(arr, 10, 12);
    mutated |= logger.cond_swap_gt(arr, 11, 13);
    // Stage 10: merge sorted 8s → sorted 16 (fixup 2)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 7, 8);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 11, 12);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    mutated
}
