use crate::sorts::quick_sorts::strategy_registry::{
    get_partitions, get_pivots, make_quick_sort_name, make_quick_sort_optimized_name,
};

#[ctor::ctor]
fn register_quick_sort_combinations() {
    // Generate and register all combinations of partition and pivot strategies
    let partitions = get_partitions();
    let pivots = get_pivots();

    for partition in partitions.iter() {
        for pivot in pivots.iter() {
            let name = make_quick_sort_name(partition, pivot);
            sort_registry_core::register_sort(&name, "O(N Log(N))", false, "quick_sorts");

            let opt_name = make_quick_sort_optimized_name(partition, pivot);
            sort_registry_core::register_sort(&opt_name, "O(N Log(N))", false, "quick_sorts");
        }
    }
}
