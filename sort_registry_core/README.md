# sort_registry_core

Shared runtime state for sort discovery. Owns the global list of registered sort names and the `register_sort` function that populates it.

## What it contains

- **`SORT_NAMES`** — a `Mutex<Vec<String>>` holding the name of every sort that has registered itself at startup.
- **`register_sort(name, big_o, stable, category)`** — called by each sort's `#[ctor]` initialiser to announce its existence. Deduplicates by name.
- **`get_registered_sorts()`** — returns a snapshot of all registered names; used by the CLI to build the sort selection menu.
- **`SortRegistry` trait** — implemented (via derive macro) by each sort type. Provides a `register()` method that inserts the sort's function pointer into `SORT_REGISTRY` and calls `register_sort`.

## Why it's a separate crate

The registry state has to be visible to both the proc macro crate (`sort_registry_macro`) and the root crate simultaneously. Proc macro crates cannot export runtime items, so the shared state lives here and both crates depend on it. This also keeps the `lazy_static` dependency out of the sort implementations themselves.
