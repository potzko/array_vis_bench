/// Shell sort + shell-shell sort registration.
///
/// Iterates both distributed slices at startup and registers every variant
/// into `SORT_REGISTRY` and `SORT_NAMES`.  No changes needed here when
/// adding new variants — edit `sequences.rs` or `branching.rs` only.
#[ctor::ctor]
fn register_shell_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();

    for entry in crate::sorts::shell_sorts::sequences::GAP_SEQUENCES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort(entry.name, entry.big_o, false, "shell_sorts");
    }

    for entry in crate::sorts::shell_sorts::branching::BRANCHING_STRATEGIES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort(entry.name, entry.big_o, false, "shell_shell_sorts");
    }
}
