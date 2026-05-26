use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, forward_block_swap, backward_block_swap};

/// Gries-Mills rotation (1981).
pub struct GriesMillsRotation;

impl Rotation for GriesMillsRotation {
    const NAME: &'static str = "gries-mills";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let mut left = split_ind;
        let mut right = n - left;
        let mut start = 0usize;
        while left != 0 && right != 0 {
            if left <= right {
                loop {
                    forward_block_swap(arr, start, start + left, left, logger);
                    start += left;
                    right -= left;
                    if left > right {
                        break;
                    }
                }
            } else {
                loop {
                    backward_block_swap(arr, start + left - right, start + left, right, logger);
                    left -= right;
                    if right > left {
                        break;
                    }
                }
            }
        }
    }
}

impl array_vis_bench_traits::HasSpace for GriesMillsRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for GriesMillsRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for GriesMillsRotation {
    const STABLE: bool = false;
}
