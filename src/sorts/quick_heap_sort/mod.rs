pub mod deferred_quick_heap_sort;
pub mod quick_heap_sort;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/quick_heap_sort_combinations.rs"));
}

