use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, reverse};

/// Triple-reversal rotation (pre-1981).
pub struct ReversalRotation;

impl Rotation for ReversalRotation {
    const NAME: &'static str = "reversal";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if split_ind == 0 || split_ind == n {
            return;
        }
        reverse(&mut arr[..split_ind], logger);
        reverse(&mut arr[split_ind..], logger);
        reverse(arr, logger);
    }
}

impl array_vis_bench_traits::HasSpace for ReversalRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for ReversalRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for ReversalRotation {
    const STABLE: bool = false;
}
