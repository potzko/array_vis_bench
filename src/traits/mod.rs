pub mod log_traits;
pub mod sort_traits;

pub use log_traits::*;
pub use sort_traits::*;

/// Type alias kept for ergonomics inside per-family `register_*!` macros
/// (`SortFn` is a self-explanatory name at the call site even though the
/// shape is trivial). Not a registry key any more — algorithm dispatch
/// goes through `bench_registry::ALGORITHMS`.
pub type SortFn = fn(&mut [usize], &mut log_traits::NoOpLogger);

/// Get all registered sort names (from core)
pub fn get_registered_sorts() -> Vec<String> {
    sort_registry_core::get_registered_sorts()
}

/// Build the full navigation tree for the interactive sort-selection menu.
pub fn get_sort_tree() -> sort_registry_core::SortTree {
    sort_registry_core::get_sort_tree()
}
