pub mod pivot_selectors;
pub mod partitions;
pub mod partitions_standalone;
pub mod quick_sort;
pub mod dual_pivot_quick_sort;
pub mod deferred_quick_sort;
pub mod deferred_dual_pivot_quick_sort;
pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
}
