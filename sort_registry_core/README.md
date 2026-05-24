# sort_registry_core

Navigation-tree state shared between every algorithm registration and the interactive picker. No runtime function pointers live here — those are in `bench_registry::ALGORITHMS` in the root crate.

## What it contains

- **`SORT_ENTRIES`** — a `Mutex<Vec<(name, big_o, stable, path)>>` populated at startup as each algorithm's per-leaf `#[ctor]` calls `register_sort_path`.
- **`register_sort_path(name, big_o, stable, path)`** — the entry point every registration macro reaches for. Appends one entry; deduplicates by name.
- **`get_registered_sorts()`** — returns names in depth-first menu order (subtree-size first), used by the interactive CLI and the `bench_registry::sorted()` ordering pass.
- **`get_sort_tree() -> SortTree`** — builds the hierarchical navigation tree from the recorded paths. The interactive binary walks this tree to drive its category → family → variant menu.
- **`registered_path_entries()`** — flat `(name, path)` pairs, consumed by `bench_registry::validate_registries` to catch duplicate tree paths.

## Why it's a separate crate

The metadata has to be visible to multiple registration macros (the proc-macro `sort_registry_macro`, the build-script-driven `combo_codegen`, and the category-specific `register_*!` macros in the root crate) without any of them depending on each other. A small leaf crate with no project dependencies serves all of them.

This also keeps the `lazy_static` dependency confined here instead of leaking into the sort implementations.
