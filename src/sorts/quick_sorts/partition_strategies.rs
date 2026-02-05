use crate::sorts::quick_sorts::strategy_registry::register_partition_strategy;

#[ctor::ctor]
fn register_partitions() {
    // User-requested partition strategies
    register_partition_strategy("partition_left_left");
    register_partition_strategy("partition_left_right_pointers");
}
