/// Comb sort registration.
///
/// Iterates the COMB_SEQUENCES distributed slice at startup and registers every
/// variant into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `sequences.rs` only.
#[ctor::ctor]
fn register_comb_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();

    for entry in crate::sorts::comb_sorts::sequences::COMB_SEQUENCES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, entry.path);
    }
}
