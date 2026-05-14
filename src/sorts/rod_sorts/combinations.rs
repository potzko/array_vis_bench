/// Rod sort registration.
///
/// Iterates the ROD_STRATEGIES distributed slice at startup and registers every
/// variant into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `branching.rs` only.
#[ctor::ctor]
fn register_rod_sorts() {
    for entry in crate::sorts::rod_sorts::branching::ROD_STRATEGIES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
