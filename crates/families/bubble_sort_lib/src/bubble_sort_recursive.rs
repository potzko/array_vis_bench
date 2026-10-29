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
            logger.cond_swap_lt(arr, i, i - 1);
        }
        let len = arr.len();
        Self::sort(&mut arr[..len - 1], logger);
    }
}

array_vis_bench_traits::direct_sort!(BubbleSortRecursive);

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;
    use sort_logger::NoOpLogger;

    // Orders by `key` alone, so `tag` shows where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Keyed { key: usize, tag: usize }
    impl PartialEq for Keyed { fn eq(&self, other: &Self) -> bool { self.key == other.key } }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
    impl Ord for Keyed { fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) } }

    fn tags_after_sort(keys: &[usize]) -> Vec<usize> {
        let mut arr: Vec<Keyed> = keys.iter().enumerate().map(|(tag, &key)| Keyed { key, tag }).collect();
        BubbleSortRecursive::sort(&mut arr, &mut NoOpLogger);
        arr.iter().map(|k| k.tag).collect()
    }

    #[test]
    fn equal_keys_keep_their_input_order() {
        assert_eq!(tags_after_sort(&[0, 0, 0, 0]), [0, 1, 2, 3]);
        assert_eq!(tags_after_sort(&[1, 0, 1, 0, 1, 0]), [1, 3, 5, 0, 2, 4]);
    }
}
