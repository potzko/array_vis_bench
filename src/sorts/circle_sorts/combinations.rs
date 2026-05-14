/// Circle sort menu-tree registration.
///
/// Iterates `CIRCLE_ENTRIES` at startup and registers each variant's path
/// with `sort_registry_core` so the interactive menu can navigate to it.
/// Algorithm dispatch itself happens through `bench_registry::ALGORITHMS`
/// (populated by the same per-variant entry in `sequences.rs`).
#[ctor::ctor]
fn register_circle_sorts() {
    for entry in crate::sorts::circle_sorts::sequences::CIRCLE_ENTRIES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
