use crate::sorts::quick_sorts::strategy_registry::register_pivot_strategy;

#[ctor::ctor]
fn register_pivots() {
    // User-requested pivot strategies
    register_pivot_strategy("first_element");
    register_pivot_strategy("last_element");
    register_pivot_strategy("middle_element");
    register_pivot_strategy("median_of_three");
    register_pivot_strategy("first_three");
    register_pivot_strategy("three_last");
    register_pivot_strategy("median_of_medians");
}
