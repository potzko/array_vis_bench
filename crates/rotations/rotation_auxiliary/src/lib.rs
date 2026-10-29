use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{
    buf_rotate_left, buf_rotate_left_using, buf_rotate_right, buf_rotate_right_using, Rotation,
};

/// Auxiliary rotation: copy the smaller side to a buffer (2021). The smaller
/// side of any split of `n` is at most `n / 2`, so that is the scratch it
/// asks for: a caller allocates it once and every rotation copies through
/// it. A caller that hands over less gets a buffer allocated for the call.
pub struct AuxiliaryRotation;

impl Rotation for AuxiliaryRotation {
    const NAME: &'static str = "auxiliary";

    #[inline]
    fn scratch_size(n: usize) -> usize {
        n / 2
    }

    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let left = split_ind;
        let right = n - left;
        if left == 0 || right == 0 {
            return;
        }
        let fits = left.min(right) <= scratch.len();
        match (left <= right, fits) {
            (true, true) => buf_rotate_left_using(arr, left, scratch, logger),
            (false, true) => buf_rotate_right_using(arr, left, scratch, logger),
            (true, false) => buf_rotate_left(arr, left, logger),
            (false, false) => buf_rotate_right(arr, left, logger),
        }
    }
}

impl array_vis_bench_traits::HasSpace for AuxiliaryRotation {
    /// Copies the smaller side into a buffer of at most N/2.
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::N1;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// catalog as a first-class rotation through this crate's spec fragment;
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

#[cfg(test)]
mod tests {
    use super::*;
    use sort_logger::{SortLog, VisualizerLogger};

    fn logger() -> VisualizerLogger<u32> {
        VisualizerLogger { type_ghost: std::marker::PhantomData, log: Vec::new() }
    }

    fn allocations(log: &VisualizerLogger<u32>) -> usize {
        log.log.iter().filter(|e| matches!(e, SortLog::CreateAuxArrT { .. })).count()
    }

    fn rotated(n: u32, split: usize) -> Vec<u32> {
        let mut want: Vec<u32> = (0..n).collect();
        want.rotate_left(split);
        want
    }

    #[test]
    fn every_split_rotates_through_the_scratch_without_allocating() {
        for n in 0..40u32 {
            for split in 0..=n as usize {
                let mut arr: Vec<u32> = (0..n).collect();
                let mut scratch = vec![0; AuxiliaryRotation::scratch_size(n as usize)];
                let mut log = logger();
                AuxiliaryRotation::rotate(&mut arr, split, &mut scratch, &mut log);
                assert_eq!(arr, rotated(n, split), "n={n} split={split}");
                assert_eq!(allocations(&log), 0, "n={n} split={split}");
            }
        }
    }

    #[test]
    fn a_scratch_too_small_falls_back_to_a_buffer_of_its_own() {
        let mut arr: Vec<u32> = (0..10).collect();
        let mut log = logger();
        AuxiliaryRotation::rotate(&mut arr, 3, &mut [], &mut log);
        assert_eq!(arr, rotated(10, 3));
        assert_eq!(allocations(&log), 1);
        assert!(matches!(log.log.last(), Some(SortLog::FreeAuxArr { .. })));
    }
}
