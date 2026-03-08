/// Shell sort registration.
///
/// A single `#[ctor::ctor]` iterates the `GAP_SEQUENCES` distributed slice
/// (populated in `sequences.rs`) and registers each variant into
/// `SORT_REGISTRY` and `SORT_NAMES`.
///
/// To add a new shell-sort variant, add it to `sequences.rs` only —
/// no changes to this file are needed.
#[ctor::ctor]
fn register_shell_sorts() {
    for entry in crate::sorts::shell_sorts::sequences::GAP_SEQUENCES {
        crate::traits::SORT_REGISTRY
            .lock()
            .unwrap()
            .insert(entry.name.to_string(), entry.sort_fn);

        sort_registry_core::register_sort(entry.name, entry.big_o, false, "shell_sorts");
    }
}
