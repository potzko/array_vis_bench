use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::NonTrivialSmallSort;

pub struct OddEvenBubbleSort<S: NonTrivialSmallSort> {
    _phantom: PhantomData<S>,
}

impl<S: NonTrivialSmallSort> OddEvenBubbleSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mut mutated = true;
        while mutated {
            mutated = false;
            // Even pass: blocks starting at 0, T, 2T, …. The last block is
            // clamped to arr.len() so a tail shorter than T still gets sorted.
            for start in (0..arr.len()).step_by(S::THRESHOLD) {
                let end = (start + S::THRESHOLD).min(arr.len());
                mutated |= S::sort(&mut arr[start..end], logger);
            }
            // Odd pass: blocks offset by T/2 — overlaps the even-pass blocks
            // at every boundary so adjacent pairs converge.
            for start in ((S::THRESHOLD / 2)..arr.len()).step_by(S::THRESHOLD) {
                let end = (start + S::THRESHOLD).min(arr.len());
                mutated |= S::sort(&mut arr[start..end], logger);
            }
        }
    }
}

combo_codegen::sort_family!(
    type = OddEvenBubbleSort<{S}>,
    uses = [
        "crate::utils::small_sort::{Size2SmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
        "crate::sorts::bubble_sorts::odd_even_bubble_sort::OddEvenBubbleSort",
    ],
    S: NonTrivialSmallSort,
    name = "odd-even bubble sort",
    big_o = "O(N^2)",
    stable = true,
    direct_sort = true,
    path = ["bubble sorts", "odd-even bubble sort", "{S}"],
);
