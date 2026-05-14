/// Shell sort registration.
///
/// Iterates the GAP_SEQUENCES distributed slice at startup and registers every
/// variant into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `sequences.rs` only.
#[ctor::ctor]
fn register_shell_sorts() {
    for entry in crate::sorts::shell_sorts::sequences::GAP_SEQUENCES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
