use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{Rotation, forward_block_swap};

/// Conjoined Triple Reversal (contrev, 2021).
pub struct ContrevRotation;

impl Rotation for ContrevRotation {
    const NAME: &'static str = "contrev";
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
        if left == right {
            forward_block_swap(arr, 0, left, left, logger);
            return;
        }
        let mut pta = 0usize;
        let mut ptb = left;
        let mut ptc = left;
        let mut ptd = n;
        if left > right {
            let mut cnt = right / 2;
            while cnt > 0 {
                ptb -= 1; ptd -= 1;
                let (a, b, c, d) = (arr[pta], arr[ptb], arr[ptc], arr[ptd]);
                logger.write_data(arr, ptb, a);
                logger.write_data(arr, pta, c);
                logger.write_data(arr, ptc, d);
                logger.write_data(arr, ptd, b);
                pta += 1; ptc += 1;
                cnt -= 1;
            }
            let mut cnt = (ptb - pta) / 2;
            while cnt > 0 {
                ptb -= 1; ptd -= 1;
                let (a, b, d) = (arr[pta], arr[ptb], arr[ptd]);
                logger.write_data(arr, ptb, a);
                logger.write_data(arr, pta, d);
                logger.write_data(arr, ptd, b);
                pta += 1;
                cnt -= 1;
            }
            let mut cnt = (ptd - pta) / 2;
            while cnt > 0 {
                ptd -= 1;
                logger.swap(arr, pta, ptd);
                pta += 1;
                cnt -= 1;
            }
        } else {
            // left < right
            let mut cnt = left / 2;
            while cnt > 0 {
                ptb -= 1; ptd -= 1;
                let (a, b, c, d) = (arr[pta], arr[ptb], arr[ptc], arr[ptd]);
                logger.write_data(arr, ptb, a);
                logger.write_data(arr, pta, c);
                logger.write_data(arr, ptc, d);
                logger.write_data(arr, ptd, b);
                pta += 1; ptc += 1;
                cnt -= 1;
            }
            let mut cnt = (ptd - ptc) / 2;
            while cnt > 0 {
                ptd -= 1;
                let (a, c, d) = (arr[pta], arr[ptc], arr[ptd]);
                logger.write_data(arr, ptc, d);
                logger.write_data(arr, ptd, a);
                logger.write_data(arr, pta, c);
                ptc += 1; pta += 1;
                cnt -= 1;
            }
            let mut cnt = (ptd - pta) / 2;
            while cnt > 0 {
                ptd -= 1;
                logger.swap(arr, pta, ptd);
                pta += 1;
                cnt -= 1;
            }
        }
    }
}

impl array_vis_bench_traits::HasSpace for ContrevRotation {
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for ContrevRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for ContrevRotation {
    const STABLE: bool = false;
}
