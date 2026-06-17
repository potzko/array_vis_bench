use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct ShakerSort;

// Composable annotations (spec compiler inherits these). This implementation has
// no early-exit, so it is Θ(N²) on every input; in-place; stable.
impl HasTimeBounds for ShakerSort {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_SQUARED;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl HasSpace for ShakerSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for ShakerSort {
    const STABLE: bool = true;
}

impl ShakerSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        let mut left = 0;
        let mut right = arr.len() - 1;
        while left < right {
            for i in left + 1..=right {
                logger.cond_swap_lt(arr, i, i - 1);
            }
            right -= 1;
            for i in (left + 1..=right).rev() {
                logger.cond_swap_lt(arr, i, i - 1);
            }
            left += 1;
        }
    }
}

// Legacy self-registration — gated OFF by default (spec is the registrar).
#[cfg(feature = "self_register")]
sort_registry_macro::sort_family! {
    type Sort = ShakerSort;
    name        = "shaker sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "shaker sort"];
}
