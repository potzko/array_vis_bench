//! Heap sort family — `HeapSort<HA>` marker + the `NaryHeapSort<H, DH>`
//! stateless-heap adapter + the supporting trait layers (`HeapLayout`,
//! `Heap`, `HeapAlgorithm`, `Compare`, `Layout`, `Direction`, `Arity`,
//! `DeepHeapify`, `HeapPartition`). Three sort families self-register via
//! TOML declarations in this crate's `Cargo.toml`:
//!
//! - `HeapSort<NaryHeapSort<ArityHeap<A, D>, DH>>` — classic heap sort
//! - `HeapSort<NaryHeapSort<ArityHeap<A, D>, QDH<HP, V>>>` — quickselect-build
//! - `HeapSort<NaryHeapSort<ArityHeap<A, D>, DPQDH<DPS>>>` — dual-pivot build
//!
//! Component metadata for `Arity`, `HeapDirection`, `HeapPartition`,
//! `DeepHeapify`, `QuickDeepHeapify`, `DualPivotQuickDeepHeapify` lives
//! in `heap_internals_components`.

pub mod arity;
pub mod arity_heap;
pub mod compare;
pub mod deep_heapify;
pub mod direction;
pub mod heap;
pub mod heap_algorithm;
pub mod heap_partition;
pub mod heap_sort;
pub mod heap_sort_quick_build;
pub mod layout;
pub mod quick_deep_heapify;
pub mod set_quick_select;
pub mod spec_drivers;

pub use arity::{Arity, Base16, Base256, Binary, Ternary};
pub use arity_heap::ArityHeap;
pub use compare::{Compare, Max, Min};
pub use deep_heapify::{DeepHeapify, Iterative, Recursive};
pub use direction::{Direction, MaxForward, MaxReverse, MinForward, MinReverse};
pub use heap::{Heap, HeapLayout};
pub use heap_algorithm::HeapAlgorithm;
pub use heap_partition::{Block, HeapPartition, LeftRightPartition, LeftLeftPartition};
pub use heap_sort::{HeapSort, NaryHeapSort};
pub use layout::{Forward, Layout, Reverse};
pub use quick_deep_heapify::{
    RecursivePartialQuickDeepHeapify, SequentialQuickDeepHeapify,
    StackDualPivotPartialQuickDeepHeapify, StackPartialQuickDeepHeapify,
};
pub use set_quick_select::{RecursiveSet, SequentialSet, SetQuickSelect, StackSet};
pub use spec_drivers::{HeapSortClassicOf, HeapSortDualBuildOf, HeapSortQuickBuildOf};
