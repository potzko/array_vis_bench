use array_vis_bench_traits::SortAlgo;
use sort_logger::SortLogger;

use crate::comb_sort::CombSort;

/// A comb sort parameterised by a rational shrink factor `NUM / DEN`.
///
/// At each step the gap is multiplied by `DEN / NUM` (i.e. divided by
/// the shrink factor). The classic 1.3 factor is represented as
/// `CombSortRatio<10, 13>` (gap = gap * 10 / 13).
///
/// `NUM` is the numerator and `DEN` is the denominator of the
/// **reciprocal** shrink factor, so `gaps_next = gap * NUM / DEN`.
pub struct CombSortRatio<const NUM: usize, const DEN: usize>;

impl<const NUM: usize, const DEN: usize> CombSortRatio<NUM, DEN> {
    fn gaps(n: usize) -> Vec<usize> {
        let mut g = n;
        let mut gs = Vec::new();
        while g > 1 {
            g = (g * NUM / DEN).max(1);
            gs.push(g);
        }
        if gs.last() != Some(&1) {
            gs.push(1);
        }
        gs
    }
}

impl<T: Ord + Copy, U: SortLogger<T>, const NUM: usize, const DEN: usize> SortAlgo<T, U>
    for CombSortRatio<NUM, DEN>
{
    fn big_o() -> &'static str {
        "O(N^2)"
    }
    fn name() -> &'static str {
        // Overridden by the registered name in the metadata family.
        "comb sort ratio"
    }
    fn sort(arr: &mut [T], logger: &mut U) {
        CombSort::sort_with_gaps(arr, logger, Self::gaps(arr.len()));
    }
    fn stable() -> bool {
        false
    }
}
