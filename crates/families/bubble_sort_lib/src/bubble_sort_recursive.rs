use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct BubbleSortRecursive;

// Composable annotations (spec compiler inherits these). No early-exit, so Θ(N²)
// on every input; in-place; stable.
impl HasTimeBounds for BubbleSortRecursive {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_SQUARED;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl HasSpace for BubbleSortRecursive {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for BubbleSortRecursive {
    const STABLE: bool = true;
}

impl BubbleSortRecursive {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        for i in 1..arr.len() {
            logger.cond_swap_le(arr, i, i - 1);
        }
        let len = arr.len();
        Self::sort(&mut arr[..len - 1], logger);
    }
}

// Legacy self-registration — gated OFF by default (spec is the registrar).
#[cfg(feature = "self_register")]
sort_registry_macro::sort_family! {
    type Sort = BubbleSortRecursive;
    name        = "bubble sort recursive";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "bubble sort recursive"];
}
