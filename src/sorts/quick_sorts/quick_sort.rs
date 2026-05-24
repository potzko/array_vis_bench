use std::marker::PhantomData;

use crate::traits::complexity::Complexity;
use crate::traits::composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
use crate::utils::small_sort::SmallSort;
use crate::traits::log_traits::SortLogger;

use super::partitions::PartitionScheme;
use super::pivot_selectors::PivotSelector;

combo_codegen::family!(
    type = QuickSort<{P}, {V}, {SS}>,
    uses = [
        "crate::utils::small_sort::{InsertionSmallSort, Network16SmallSort, NetworkSmallSort, NoSmallSort, Size1SmallSort, Size2SmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay}",
        "super::partitions::MovingPivotV3",
        "super::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther, CombinedSelector, NintherDualPivot}",
        "super::quick_sort::QuickSort",
        "crate::utils::rotation::ReversalRotation",
    ],
    P: Partition,
    V: PivotSelector,
    SS: SmallSort,
    name = "quick sort classic",
    big_o = inherited,
    space = inherited,
    stable = inherited,
    adaptive = false,
    direct_sort = true,
    path = ["quick sorts", "classic", "{P}", "{V}", "{SS}"],
);

pub struct QuickSort<P: PartitionScheme, V: PivotSelector, SS: SmallSort>(
    PhantomData<(P, V, SS)>,
);

impl<P: PartitionScheme, V: PivotSelector, SS: SmallSort> QuickSort<P, V, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        quick_sort_recursive::<T, U, P, V, SS>(arr, logger);
    }
}

fn quick_sort_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
    SS: SmallSort,
>(
    arr: &mut [T],
    logger: &mut U,
) {
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot_idx);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[..left_end], logger);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[right_start..], logger);
}

// ── Composable annotations ──────────────────────────────────────────
//
// Per-level work: partition + pivot selection. The small-sort slot
// contributes O(1) to QuickSort's overall complexity because it only
// runs on bounded-size slices (`arr.len() <= SS::THRESHOLD`) — so the
// composition ignores `SS::WORST` and uses `Complexity::CONST` for that
// slot, even when the SS algorithm is intrinsically O(N²).
//
// Worst case: degenerate pivots collapse to O(N) recursion depth →
//             O(N · per-level). Median-of-medians keeps it at O(log N).
// Best / average: O(log N) depth assuming balanced partitions.

impl<P, V, SS> HasTimeBounds for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasTimeBounds,
    V: PivotSelector + HasTimeBounds + PivotQuality,
    SS: SmallSort,
{
    /// Recursion depth × per-level (partition + pivot) × small-sort (O(1)).
    /// Depth is O(N) if the pivot can degenerate, else O(log N).
    const WORST: Complexity = Complexity::product(
        if V::DEGENERATES { Complexity::N1 } else { Complexity::LOG_N },
        Complexity::sum(P::WORST, V::WORST),
    );
    /// Balanced split → O(log N) depth.
    const BEST: Complexity = Complexity::product(
        Complexity::LOG_N,
        Complexity::sum(P::BEST, V::BEST),
    );
    const AVERAGE: Complexity = Complexity::product(
        Complexity::LOG_N,
        Complexity::sum(P::AVERAGE, V::AVERAGE),
    );
}

impl<P, V, SS> HasSpace for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasSpace,
    V: PivotSelector + HasSpace,
    SS: SmallSort + HasSpace,
{
    /// Recursion adds O(log N) stack baseline; take the max with each
    /// component's own aux-space contribution.
    const SPACE: Complexity = Complexity::sum(
        Complexity::LOG_N,
        Complexity::sum(P::SPACE, Complexity::sum(V::SPACE, SS::SPACE)),
    );
}

impl<P, V, SS> HasStability for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasStability,
    V: PivotSelector + HasStability,
    SS: SmallSort + HasStability,
{
    const STABLE: bool = P::STABLE && V::STABLE && SS::STABLE;
}

#[cfg(test)]
mod annotation_tests {
    use super::*;
    use crate::sorts::quick_sorts::partitions::{Lomuto, Hoare};
    use crate::sorts::quick_sorts::pivot_selectors::{FirstElement, MedianOfMedians};
    use crate::utils::small_sort::{InsertionSmallSort, LinearInsertion};

    type ClassicQS = QuickSort<Lomuto, FirstElement, InsertionSmallSort<LinearInsertion, 32>>;
    type GuaranteedQS = QuickSort<Hoare, MedianOfMedians, InsertionSmallSort<LinearInsertion, 32>>;

    #[test]
    fn worst_case() {
        // Classic Lomuto + first-pivot: degenerates on sorted input → O(N²).
        assert_eq!(ClassicQS::WORST, Complexity::N_SQUARED);
        // Median-of-medians guarantees a balanced split → O(N log N) worst.
        assert_eq!(GuaranteedQS::WORST, Complexity::N_LOG_N);
    }

    #[test]
    fn best_and_average() {
        // Both flavours: balanced split → O(N log N).
        assert_eq!(ClassicQS::BEST, Complexity::N_LOG_N);
        assert_eq!(ClassicQS::AVERAGE, Complexity::N_LOG_N);
        // Median-of-medians is itself O(N), so per-level work is O(N) and
        // depth is O(log N) → O(N log N).
        assert_eq!(GuaranteedQS::BEST, Complexity::N_LOG_N);
    }

    #[test]
    fn space() {
        // Classic: recursion stack O(log N), all aux is O(1) → O(log N).
        assert_eq!(ClassicQS::SPACE, Complexity::LOG_N);
        // Median-of-medians itself uses O(log N) stack for its own
        // recursion — same final answer.
        assert_eq!(GuaranteedQS::SPACE, Complexity::LOG_N);
    }

    #[test]
    fn in_place() {
        // Both qualify as "in-place" — only O(log N) recursion, no aux.
        assert!(ClassicQS::SPACE.is_in_place());
        assert!(GuaranteedQS::SPACE.is_in_place());
    }

    #[test]
    fn stability() {
        // Lomuto / Hoare aren't stable → composite isn't stable.
        assert!(!ClassicQS::STABLE);
        assert!(!GuaranteedQS::STABLE);
    }

    #[test]
    fn registry_entry_reflects_trait_values() {
        // End-to-end check: pick a known QuickSort combination out of the
        // global ALGORITHMS slice and assert its `worst`/`best`/`space`/
        // `stable` fields match what the trait math produced. This proves
        // the `family! { big_o = inherited, space = inherited, … }`
        // pipeline carries trait values all the way into the static entry.
        let entry = crate::bench_registry::ALGORITHMS
            .iter()
            .find(|e| {
                e.name.starts_with("quick sort classic")
                    && e.name.contains("lomuto")
                    && e.name.contains("first")
            })
            .expect("expected at least one classic-Lomuto-first QuickSort entry");
        // Lomuto + FirstElement degenerates → worst is N²;
        // BEST stays N log N because balanced partitions still yield log-depth.
        assert_eq!(entry.worst, Complexity::N_SQUARED);
        assert_eq!(entry.best, Complexity::N_LOG_N);
        assert_eq!(entry.average, Complexity::N_LOG_N);
        // Recursion stack only — in-place.
        assert_eq!(entry.space, Complexity::LOG_N);
        assert!(entry.space.is_in_place());
        assert!(!entry.stable);
        assert!(!entry.adaptive);
    }
}
