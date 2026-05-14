pub mod beap_heap;
pub mod beap_sort;
pub mod beap_sort_quick_build;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/beap_sort_combinations.rs"));
}

