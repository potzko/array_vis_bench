use lazy_static::lazy_static;
use std::sync::Mutex;

/// Trait for types that can be registered in the global sort registry (metadata side)
pub trait SortRegistry {
    /// Register this sort in the global registry
    fn register();
}

lazy_static! {
    static ref SORT_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Register sort metadata (name, complexity, stability, category)
pub fn register_sort(name: &str, _big_o: &str, _stable: bool, _category: &str) {
    let mut sort_names = SORT_NAMES.lock().unwrap();
    if !sort_names.contains(&name.to_string()) {
        sort_names.push(name.to_string());
    }
}

/// Get all registered sort names
pub fn get_registered_sorts() -> Vec<String> {
    SORT_NAMES.lock().unwrap().clone()
}
