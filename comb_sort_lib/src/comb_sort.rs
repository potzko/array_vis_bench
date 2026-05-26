use sort_logger::SortLogger;

/// Comb sort driven by a caller-supplied gap sequence.
///
/// For each gap a single forward pass is made, comparing and swapping
/// each pair `(arr[i], arr[i+gap])`. After all gaps are exhausted the
/// array is converged with repeated gap-1 passes until no swaps occur.
///
/// The gap sequence is generated per-sort by the shrink-factor closure
/// in [`registration`]. Each registered variant simply calls
/// `sort_with_gaps` with a different precomputed list.
pub struct CombSort;

impl CombSort {
    pub fn sort_with_gaps<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        gaps: Vec<usize>,
    ) {
        for gap in gaps {
            for i in 0..arr.len().saturating_sub(gap) {
                logger.cond_swap_gt(arr, i, i + gap);
            }
        }
        // Converge: repeat gap-1 passes until no swaps remain.
        loop {
            let mut swapped = false;
            for i in 0..arr.len().saturating_sub(1) {
                if logger.cond_swap_gt(arr, i, i + 1) {
                    swapped = true;
                }
            }
            if !swapped {
                break;
            }
        }
    }
}
