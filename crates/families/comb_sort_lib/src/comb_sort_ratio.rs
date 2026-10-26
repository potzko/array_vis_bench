use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
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

/// A rational shrink factor for comb sort — yields the gap schedule for a given
/// input length. Implemented by every `CombSortRatio<NUM, DEN>`; lets the spec
/// driver [`CombSortOf`] range over ratios as a single faceted slot.
pub trait CombRatio {
    fn gap_schedule(n: usize) -> Vec<usize>;
}

impl<const NUM: usize, const DEN: usize> CombRatio for CombSortRatio<NUM, DEN> {
    fn gap_schedule(n: usize) -> Vec<usize> {
        Self::gaps(n)
    }
}

/// Comb sort driven by a [`CombRatio`] shrink factor, the driver the spec
/// emits. The bare `CombSortRatio<NUM, DEN>` is a ratio, not a sort: every
/// ratio shares its head, so the query names the sort as `CombSortOf<{ratio}>`
/// and this wrapper carries the inherent `sort`.
pub struct CombSortOf<R: CombRatio>(PhantomData<R>);

impl<R: CombRatio> CombSortOf<R> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        CombSort::sort_with_gaps(arr, logger, R::gap_schedule(arr.len()));
    }
}

array_vis_bench_traits::direct_sort!(CombSortOf<R>; R: CombRatio);

// Composable annotations (spec compiler inherits these). Comb sort degrades to
// Θ(N²) in the worst case; the gap-shrinking passes give ~O(N log N) best; the
// ratio doesn't change the asymptotic class. In-place; not stable (long-range
// swaps).
impl<R: CombRatio> HasTimeBounds for CombSortOf<R> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<R: CombRatio> HasSpace for CombSortOf<R> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<R: CombRatio> HasStability for CombSortOf<R> {
    const STABLE: bool = false;
}
