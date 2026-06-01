use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, buf_rotate_left, buf_rotate_right};

/// Auxiliary rotation: copy the smaller side to a heap buffer (2021).
pub struct AuxiliaryRotation;

impl Rotation for AuxiliaryRotation {
    const NAME: &'static str = "auxiliary";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let left = split_ind;
        let right = n - left;
        if left == 0 || right == 0 {
            return;
        }
        if left <= right {
            buf_rotate_left(arr, left, logger);
        } else {
            buf_rotate_right(arr, left, logger);
        }
    }
}

impl array_vis_bench_traits::HasSpace for AuxiliaryRotation {
    /// Copies the smaller side into a heap-allocated buffer (≤ N/2).
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::N1;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for AuxiliaryRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for AuxiliaryRotation {
    const STABLE: bool = false;
}
