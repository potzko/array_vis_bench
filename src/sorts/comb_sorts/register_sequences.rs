/// Comb sort registration.
///
/// Iterates the COMB_SEQUENCES distributed slice at startup and registers every
/// variant into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `sequences.rs` only.
#[ctor::ctor]
fn register_comb_sorts() {
    for entry in crate::sorts::comb_sorts::sequences::COMB_SEQUENCES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
