use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, gcd};

/// Juggling rotation (1965): GCD cycle-based.
pub struct JugglingRotation;

impl Rotation for JugglingRotation {
    const NAME: &'static str = "juggling";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let left = split_ind;
        if left == 0 || left == n {
            return;
        }
        let cycles = gcd(left, n);
        for start in 0..cycles {
            let saved = arr[start];
            let mut pta = start;
            loop {
                let ptb = pta + left;
                let ptb = if ptb >= n { ptb - n } else { ptb };
                if ptb == start {
                    break;
                }
                let v = arr[ptb];
                logger.write_data(arr, pta, v);
                pta = ptb;
            }
            logger.write_data(arr, pta, saved);
        }
    }
}

impl array_vis_bench_traits::HasSpace for JugglingRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for JugglingRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for JugglingRotation {
    const STABLE: bool = false;
}
