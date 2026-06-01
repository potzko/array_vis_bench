use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, backward_block_swap, forward_block_swap, unit_rotate_left, unit_rotate_right};

/// Grail rotation (2020): Gries-Mills with a stack-based aux at the end.
pub struct GrailRotation;

impl Rotation for GrailRotation {
    const NAME: &'static str = "grail";
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
        let mut min = left.min(right);
        while min > 1 {
            if left <= right {
                loop {
                    forward_block_swap(arr, start, start + left, left, logger);
                    start += left;
                    right -= left;
                    if left > right {
                        break;
                    }
                }
                min = right;
            } else {
                loop {
                    backward_block_swap(arr, start + left - right, start + left, right, logger);
                    left -= right;
                    if right > left {
                        break;
                    }
                }
                min = left;
            }
        }
        // Outer-loop exit condition is `min(left, right) <= 1`, so if we
        // reach here with both > 0, exactly one of them equals 1 — a unit
        // rotation, doable fully in-place (no aux). The other side could
        // be arbitrarily large.
        if left == 1 && right > 0 {
            unit_rotate_left(&mut arr[start..start + left + right], logger);
        } else if right == 1 && left > 0 {
            unit_rotate_right(&mut arr[start..start + left + right], logger);
        }
    }
}

impl array_vis_bench_traits::HasSpace for GrailRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for GrailRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for GrailRotation {
    const STABLE: bool = false;
}
