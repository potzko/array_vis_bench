/// Circle sort registration.
///
/// Iterates `CIRCLE_ENTRIES` at startup and registers every variant into
/// `SORT_REGISTRY` and `sort_registry_core`.  No changes needed here when
/// adding new variants — edit `orderings.rs` / `directions.rs` and call the
/// appropriate macro in `sequences.rs` only.
#[ctor::ctor]
fn register_circle_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();

    for entry in crate::sorts::circle_sorts::sequences::CIRCLE_ENTRIES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, entry.path);
    }
}
