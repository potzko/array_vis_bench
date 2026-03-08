/// Shell sort registration.
///
/// Iterates the GAP_SEQUENCES distributed slice at startup and registers every
/// variant into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `sequences.rs` only.
#[ctor::ctor]
fn register_shell_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();

    for entry in crate::sorts::shell_sorts::sequences::GAP_SEQUENCES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, entry.path);
    }
}
