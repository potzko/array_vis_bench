# sort_registry_macro

Proc macro crate that eliminates registration boilerplate for sorts that use the `create_sort!` / `#[derive(SortRegistry)]` path.

## What it contains

- **`#[derive(SortRegistry)]`** — when applied to a sort's registration type, generates:
  - A monomorphic `fn __sort_fn_<name>(arr: &mut [usize], logger: &mut NoOpLogger)` — a fully inlinable function pointer stored in `SORT_REGISTRY`.
  - An `impl SortRegistry for <Type>` that resolves the sort's name, big-O, and stability from its `SortAlgo` impl, inserts the function pointer, and calls `sort_registry_core::register_sort`.
  - A `#[ctor::ctor] fn __register_<name>()` that calls `register()` at program startup — no manual wiring in `main`.

## Why it's a separate crate

Rust requires proc macro crates to be their own crate with `proc-macro = true`. The generated code references items from the root crate (`crate::traits::SORT_REGISTRY`, `crate::traits::sort_traits::SortAlgo`), so it is tightly coupled to the root but must be compiled as a separate artifact. `sort_registry_core` provides the pieces that need to be shared without pulling in the root.

## Relationship to the linkme path

Newer sorts (shell sorts, shell-shell sorts) self-register via `linkme` distributed slices and a `#[ctor]` in `combinations.rs` — they don't use this derive macro. The macro path remains for sorts that were registered before the `linkme` approach was introduced and may be phased out as the refactor progresses.
