//! Spec-system driver wrappers for the beap sort families.
//!
//! The legacy beap families all monomorphize to the shared Rust type-head
//! `HeapSort<NaryHeapSort<BeapHeap<D>, DH>>`. The spec emit resolves by Rust
//! type-head first-wins, so the three logically-distinct beap families
//! (classic / quick-build / dual-pivot) would collapse into one queryable
//! entry. These thin newtype drivers give each family a UNIQUE type-head while
//! delegating straight through to the real heap-sort chain, so the three beap
//! families stay separately queryable.
//!
//! Each driver is generic over `<D: Direction, DH: DeepHeapify>` (both from
//! `heap_sort_lib`) and forwards `sort` to
//! `HeapSort::<NaryHeapSort<BeapHeap<D>, DH>>::sort`. The composable
//! annotations forward from the inner chain so the drivers stay honest:
//! `WORST/BEST/AVERAGE = N_SQRT_N`, `SPACE = LOG_N`, `STABLE = false`.

use std::marker::PhantomData;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use heap_sort_lib::deep_heapify::DeepHeapify;
use heap_sort_lib::direction::Direction;
use heap_sort_lib::heap_sort::{HeapSort, NaryHeapSort};
use sort_logger::SortLogger;

use crate::beap_heap::BeapHeap;

/// Inner heap-sort chain shared by every beap driver. Aliased so the four
/// per-driver impls (`sort` + the three composable forwards) all name the same
/// concrete type once.
type Inner<D, DH> = NaryHeapSort<BeapHeap<D>, DH>;

macro_rules! beap_driver {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub struct $name<D: Direction, DH: DeepHeapify> {
            _phantom: PhantomData<(D, DH)>,
        }

        impl<D: Direction, DH: DeepHeapify> $name<D, DH> {
            /// Thin delegate to the real beap heap-sort chain.
            #[inline(always)]
            pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
                HeapSort::<Inner<D, DH>>::sort(arr, logger)
            }
        }

        // Forward the composable annotations from the inner chain to stay
        // honest. `NaryHeapSort<BeapHeap<D>, DH>` computes
        // WORST/BEST/AVERAGE = N_SQRT_N, SPACE = LOG_N, STABLE = false.
        impl<D: Direction, DH: DeepHeapify> HasTimeBounds for $name<D, DH> {
            const WORST: Complexity = <Inner<D, DH> as HasTimeBounds>::WORST;
            const BEST: Complexity = <Inner<D, DH> as HasTimeBounds>::BEST;
            const AVERAGE: Complexity = <Inner<D, DH> as HasTimeBounds>::AVERAGE;
        }

        impl<D: Direction, DH: DeepHeapify> HasSpace for $name<D, DH> {
            const SPACE: Complexity = <Inner<D, DH> as HasSpace>::SPACE;
        }

        impl<D: Direction, DH: DeepHeapify> HasStability for $name<D, DH> {
            const STABLE: bool = <Inner<D, DH> as HasStability>::STABLE;
        }
    };
}

beap_driver! {
    /// Driver head for the classic (iterative-build) beap sort family.
    BeapSortClassicOf
}

beap_driver! {
    /// Driver head for the single-pivot quickselect-build beap sort family.
    BeapSortQuickOf
}

beap_driver! {
    /// Driver head for the dual-pivot quickselect-build beap sort family.
    BeapSortDualOf
}
