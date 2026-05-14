pub mod log_traits;
pub mod sort_traits;

pub use log_traits::*;
pub use sort_traits::*;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

/// Function pointer type for sort implementations (fully optimizable, no trait objects)
pub type SortFn = fn(&mut [usize], &mut log_traits::NoOpLogger);

/// Function pointer type for sort visualisation (accepts dyn SortLogger)
pub type SortVisFn = fn(&mut [usize], &mut dyn log_traits::SortLogger<usize>);

lazy_static! {
    pub static ref SORT_REGISTRY: Mutex<HashMap<String, SortFn>> =
        Mutex::new(HashMap::new());

    /// Registry for visualisation dispatch — maps sort name → sort_vis fn pointer.
    /// Populated by `family!(... direct_sort = true; ...)`.
    pub static ref SORT_VIS_REGISTRY: Mutex<HashMap<String, SortVisFn>> =
        Mutex::new(HashMap::new());
}

/// Get all registered sort names (from core)
pub fn get_registered_sorts() -> Vec<String> {
    sort_registry_core::get_registered_sorts()
}

/// Build the full navigation tree for the interactive sort-selection menu.
pub fn get_sort_tree() -> sort_registry_core::SortTree {
    sort_registry_core::get_sort_tree()
}
