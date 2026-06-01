//! Bubble sort family. Three base algorithms (`BubbleSort`,
//! `ShakerSort`, `BubbleSortRecursive`) self-register via inline
//! `sort_family!` calls — no axes, single ALGORITHMS entry each.
//! `OddEvenBubbleSort<S: NonTrivialSmallSort>` is the one axis-bearing
//! variant; its family declaration is the
//! `[[package.metadata.array_vis_bench.families]]` block in this crate's
//! `Cargo.toml`.

pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

pub use bubble_sort::BubbleSort;
pub use bubble_sort_recursive::BubbleSortRecursive;
pub use odd_even_bubble_sort::OddEvenBubbleSort;
pub use shaker_sort::ShakerSort;
