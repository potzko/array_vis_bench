use std::marker::PhantomData;
use crate::traits::log_traits::SortLogger;
use super::rotation_merge::RotationMerge;
use crate::utils::small_sort::SmallSort;

combo_codegen::sort_family!(
    type = TopDownRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>,
    uses = [
        "crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort",
        "crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge}",
        "crate::utils::rotation::{ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation, TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation, HelixRotation, DrillRotation, JugglingRotation}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
    ],
    R: Rotation,
    SS: SmallSort,
    name = "rotation merge sort",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "rotation", "top-down", "{R}", "{SS}"],
);

combo_codegen::sort_family!(
    type = TopDownRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>,
    uses = [
        "crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort",
        "crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge}",
        "crate::utils::rotation::{ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation, TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation, HelixRotation, DrillRotation, JugglingRotation}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
    ],
    R: Rotation,
    SS: SmallSort,
    name = "rotation merge sort<smaller-side>",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "rotation", "top-down smaller-side", "{R}", "{SS}"],
);

combo_codegen::sort_family!(
    type = BottomUpRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>,
    uses = [
        "crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort",
        "crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge}",
        "crate::utils::rotation::{ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation, TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation, HelixRotation, DrillRotation, JugglingRotation}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
    ],
    R: Rotation,
    SS: SmallSort,
    name = "bottom-up rotation merge sort",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "rotation", "bottom-up", "{R}", "{SS}"],
);

combo_codegen::sort_family!(
    type = BottomUpRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>,
    uses = [
        "crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort",
        "crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge}",
        "crate::utils::rotation::{ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation, TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation, HelixRotation, DrillRotation, JugglingRotation}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
    ],
    R: Rotation,
    SS: SmallSort,
    name = "bottom-up rotation merge sort<smaller-side>",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "rotation", "bottom-up smaller-side", "{R}", "{SS}"],
);

/// Top-down (recursive) rotation merge sort.
///
/// Merges in-place using the `M` rotation strategy — no auxiliary array.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy (`NaiveRotation` or `SmallerSideRotation`).
/// - `EARLY_EXIT`: skip the merge when the two halves are already in order.
pub struct TopDownRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        if S::THRESHOLD > 0 && n <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mid = n / 2;
        {
            let (l, r) = arr.split_at_mut(mid);
            Self::sort(l, logger);
            Self::sort(r, logger);
        }
        if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
            return;
        }
        M::merge(arr, mid, logger);
    }
}

/// Bottom-up (iterative) rotation merge sort.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy.
/// - `EARLY_EXIT`: skip the merge when a segment pair is already sorted.
pub struct BottomUpRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }

        let gap0 = if S::THRESHOLD > 0 { S::THRESHOLD } else { 1 };

        if S::THRESHOLD > 0 {
            let mut i = 0;
            while i < n {
                let end = (i + S::THRESHOLD).min(n);
                S::sort(&mut arr[i..end], logger);
                i += S::THRESHOLD;
            }
        }

        let mut gap = gap0;
        while gap < n {
            let mut i = 0;
            while i < n {
                let mid = (i + gap).min(n);
                let end = (i + 2 * gap).min(n);
                if mid < end {
                    if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
                        // already sorted — skip
                    } else {
                        M::merge(&mut arr[i..end], mid - i, logger);
                    }
                }
                i += 2 * gap;
            }
            gap *= 2;
        }
    }
}
