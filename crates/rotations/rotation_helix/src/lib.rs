use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, unit_rotate_left, unit_rotate_right};

/// Helix rotation (2021): grail-derived, alternating inner loops.
pub struct HelixRotation;

impl Rotation for HelixRotation {
    const NAME: &'static str = "helix";
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
        let mut end = n;
        let mut mid = left;
        loop {
            if left > right {
                if right <= 1 {
                    break;
                }
                while mid > start {
                    mid -= 1;
                    end -= 1;
                    logger.swap(arr, mid, end);
                }
                left %= right;
                mid += left;
                right = end - mid;
            } else {
                if left <= 1 {
                    break;
                }
                while mid < end {
                    logger.swap(arr, mid, start);
                    mid += 1;
                    start += 1;
                }
                right %= left;
                mid -= right;
                left = mid - start;
            }
        }
        // Loop exits with `min(left, right) <= 1`. If both > 0, one is 1 —
        // a unit rotation, fully in-place.
        if left == 1 && right > 0 {
            unit_rotate_left(&mut arr[start..end], logger);
        } else if right == 1 && left > 0 {
            unit_rotate_right(&mut arr[start..end], logger);
        }
    }
}

impl array_vis_bench_traits::HasSpace for HelixRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for HelixRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for HelixRotation {
    const STABLE: bool = false;
}
