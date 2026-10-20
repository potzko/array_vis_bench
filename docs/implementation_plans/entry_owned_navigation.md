# Implementation plan: the entry owns its navigation

Status: NOT STARTED. Designed 2026-10-20 from the concept review recorded in
`../todo.md`. First of the three "entry is the single source of truth" plans,
with `entry_owned_flags.md` and `category_from_the_wrapper.md`.

## Problem

Every emitted module registers the same algorithm twice. `spec_emit` writes a
static `AlgorithmEntry` into `ALGORITHMS`, then a `#[ctor]` that calls
`sort_registry_core::register_sort_variant_structured` with the name, the menu
path, the facet axes and the label template. `sort_registry_core` keeps those
in a mutex guarded vector of `VariantDesc`, which `array_vis_bench_cli` walks
to build the picker; the CLI then looks the chosen name back up in
`ALGORITHMS`. `bench_registry::sorted` orders entries by that second list and
`validate_registries` cross checks the two for duplicate paths.

What it costs:

- 1580 constructors run before `main`, each taking the mutex.
- Two sources that have to agree, with a validator written to catch the
  cases where they do not.
- A crate whose only job is holding the second copy.
- Four legacy `register_sort_path` callers (comb, shell, circle, rod) behind
  the `self_register` features, off and enabled nowhere.

## Design

`AlgorithmEntry` gains the three things the constructor carries:

```rust
pub menu: &'static [&'static str],
pub axes: &'static [(&'static str, &'static str, &'static str)],
pub label_template: &'static str,
```

The axes triple is `(role, value, path)`, exactly what `AxisBinding` holds.
`spec_emit` already builds `menu_lit`, `axes_lit` and `template_lit` for the
constructor call; it writes them into the static instead and the `#[ctor]`
block goes.

`VariantDesc`, `AxisBinding` and `SortTree` move into `array_vis_bench_cli`,
their only consumer, as a view built from `ALGORITHMS` on demand. `sorted`
orders by `(menu, axes)` read from the entries. The tree path check in
`validate_registries` reads the entries too. `sort_registry_core` is deleted,
and with it the four `self_register` features (todo item 8).

## Steps

1. The bridge test, in `spec_catalog`: for every entry, the `VariantDesc` the
   picker holds under that name equals the entry's `menu`, `axes` and
   `label_template`. It does not compile until the fields exist and passes
   once the emit fills them, while the constructor is still there.
2. Add the fields and fill them in all seven emit arms. The `spec_emit` tests
   that assert on the `register_sort_variant_structured` text assert on the
   fields instead.
3. Move the view types into the CLI with a `variants()` function over
   `ALGORITHMS`. The CLI tests that build a `VariantDesc` by hand keep
   working unchanged.
4. Point `sorted` and `validate_registries` at the entries, delete the
   constructor from the emit, delete the bridge test's second source.
5. Delete the `self_register` features and their callers, then the crate:
   workspace members, `AbiPaths.nav` in `spec_emit`, the `ctor` dependency
   where nothing else uses it, and the docs.

## Checks

- `catalog.golden` and the entry fingerprint are unchanged throughout.
- No `#[ctor]` remains in `catalog_entries.rs`; the two constructors in
  `bench_registry` (validation and subprocess dispatch) are the only ones
  left.
- `grep -rn sort_registry_core` over the tree finds nothing.
- Time `cargo run --release -- --timing 1 1` before and after; the
  constructor storm is the expected saving.
