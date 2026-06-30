//! Bubble sort family — `BubbleSort`, `ShakerSort`, `BubbleSortRecursive`
//! (no axes) and `OddEvenBubbleSort<S: NonTrivialSmallSort>` (one axis). This
//! crate provides the algorithm types only; all variants are emitted into
//! `array_vis_bench_core::ALGORITHMS` by the spec catalog from this crate's
//! `bubble.spec`/`bubble.avbs` fragment.

pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

pub use bubble_sort::BubbleSort;
pub use bubble_sort_recursive::BubbleSortRecursive;
pub use odd_even_bubble_sort::OddEvenBubbleSort;
pub use shaker_sort::ShakerSort;
