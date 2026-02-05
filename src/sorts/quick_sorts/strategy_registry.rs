use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static! {
    static ref PARTITION_STRATEGIES: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
    static ref PIVOT_STRATEGIES: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
}

pub fn register_partition_strategy(name: &'static str) {
    PARTITION_STRATEGIES.lock().unwrap().insert(name);
}

pub fn register_pivot_strategy(name: &'static str) {
    PIVOT_STRATEGIES.lock().unwrap().insert(name);
}

pub fn get_partitions() -> Vec<&'static str> {
    PARTITION_STRATEGIES.lock().unwrap().iter().copied().collect()
}

pub fn get_pivots() -> Vec<&'static str> {
    PIVOT_STRATEGIES.lock().unwrap().iter().copied().collect()
}

pub fn make_quick_sort_name(partition: &str, pivot: &str) -> String {
    format!(
        "quick_sort<partition: {}<pivot_selection: {}>>",
        partition, pivot
    )
}

pub fn make_quick_sort_optimized_name(partition: &str, pivot: &str) -> String {
    format!(
        "quick_sort_optimized<partition: {}<pivot_selection: {}>>",
        partition, pivot
    )
}
