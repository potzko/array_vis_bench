//! `HeapAlgorithmPair<DH>` — pick the two opposite-direction
//! [`HeapAlgorithm`] instances that [`crate::heap_partition_core::build_and_converge`]
//! drives, parameterised by the deep-heapify strategy.
//!
//! The build-and-converge dance needs a *left* heap rooted at the low end
//! (max-forward orientation, so the running maximum is at index `0`) and a
//! *right* heap rooted at the high end (min-reverse orientation, so the
//! running minimum is at index `n-1`). Both wrap the same underlying
//! "heap kind" (d-ary, beap, …) with opposing [`Direction`]s.
//!
//! Implementations:
//!
//! - [`AryPair<A>`] — `NaryHeapSort<ArityHeap<A, MaxForward>, DH>` /
//!   `NaryHeapSort<ArityHeap<A, MinReverse>, DH>`. The default d-ary
//!   variant, generic over [`Arity`] (Binary / Ternary / Base16 / Base256).
//! - [`BeapPair`] — `NaryHeapSort<BeapHeap<MaxForward>, DH>` /
//!   `NaryHeapSort<BeapHeap<MinReverse>, DH>`. Bi-parental heap; no
//!   arity parameter.
//!
//! Slots into [`crate::heap_extract::HeapExtract`] as its `P` parameter,
//! and into [`crate::heap_partition_core::build_and_converge`] for both
//! `QuickHeapSort` (which always uses [`AryPair`]) and `HeapExtract`
//! (which surfaces both pair kinds through metadata).

use std::marker::PhantomData;

use beap_sort_lib::beap_heap::BeapHeap;
use heap_sort_lib::arity::Arity;
use heap_sort_lib::arity_heap::ArityHeap;
use heap_sort_lib::deep_heapify::DeepHeapify;
use heap_sort_lib::direction::{MaxForward, MinReverse};
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use heap_sort_lib::heap_sort::NaryHeapSort;

/// A pair of opposite-direction [`HeapAlgorithm`]s used by
/// [`crate::heap_partition_core::build_and_converge`]. The `Left` heap is
/// max-rooted at the low end, the `Right` heap is min-rooted at the high end
/// — the two sides whose head-swap convergence forms the heap-extract
/// partition.
pub trait HeapAlgorithmPair<DH: DeepHeapify> {
    type Left: HeapAlgorithm;
    type Right: HeapAlgorithm;
}

/// D-ary heap pair (`NaryHeapSort<ArityHeap<A, _>, DH>`). The default
/// heap-extract pair shape — what `QuickHeapSort` has always used.
pub struct AryPair<A: Arity> {
    _phantom: PhantomData<A>,
}

impl<A: Arity, DH: DeepHeapify> HeapAlgorithmPair<DH> for AryPair<A> {
    type Left = NaryHeapSort<ArityHeap<A, MaxForward>, DH>;
    type Right = NaryHeapSort<ArityHeap<A, MinReverse>, DH>;
}

/// Bi-parental heap pair (`NaryHeapSort<BeapHeap<_>, DH>`). No arity
/// parameter — beap structure is fixed.
pub struct BeapPair;

impl<DH: DeepHeapify> HeapAlgorithmPair<DH> for BeapPair {
    type Left = NaryHeapSort<BeapHeap<MaxForward>, DH>;
    type Right = NaryHeapSort<BeapHeap<MinReverse>, DH>;
}
