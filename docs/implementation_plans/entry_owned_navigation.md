# Implementation plan: the entry owns its navigation

Status: DONE (2026-10-20). Designed from the concept review recorded in
`../todo.md`, first of the three "entry is the single source of truth" plans,
with `entry_owned_flags.md` and `category_from_the_wrapper.md`. Built as
designed, with one correction found after the design: the view types have two
consumers, the picker and `bench_registry::sorted`, so they live in
`array_vis_bench_core::navigation` and not in the CLI. The catalog golden and
the entry fingerprint did not move.

## Problem

Every emitted module registered the same algorithm twice. `spec_emit` wrote a
static `AlgorithmEntry` into `ALGORITHMS`, then a `#[ctor]` that called the
structured registration function of the navigation registry crate with the
name, the menu path, the facet axes and the label template. That crate kept
them in a mutex guarded vector of `VariantDesc`, which `array_vis_bench_cli`
walked to build the picker; the CLI then looked the chosen name back up in
`ALGORITHMS`. `bench_registry::sorted` ordered entries by that second list and
`validate_registries` cross checked the two for duplicate paths.

What it cost:

- 1580 constructors ran before `main`, each taking the mutex.
- Two sources that had to agree, with a validator written to catch the
  cases where they did not.
- A crate whose only job was holding the second copy.
- Four legacy callers of its flat path registration (comb, shell, circle,
  rod) behind self registration features, off and enabled nowhere.

## Design

`AlgorithmEntry` gains the three things the constructor carried:

```rust
pub menu: &'static [&'static str],
pub axes: &'static [(&'static str, &'static str, &'static str)],
pub label_template: &'static str,
```

The axes triple is `(role, value, path)`, exactly what `AxisBinding` holds.
`spec_emit` already built `menu_lit`, `axes_lit` and `template_lit` for the
constructor call; it writes them into the static instead and the `#[ctor]`
block is gone.

`VariantDesc`, `AxisBinding` and `SortTree` moved into
`array_vis_bench_core::navigation`, as a view built from `ALGORITHMS` on
demand. `variants()` lists one `VariantDesc` per entry for the picker.
`SortTree::of` nests entries by menu path and axis values, children by subtree
size then name and leaves by label then name, and `sorted` reads its depth
first order. `repeated_name_errors` is the tree path check in
`validate_registries`. The navigation registry crate is deleted, and with it
the four self registration features (todo item 8).

## Steps

1. The bridge test, in `spec_catalog`: for every entry, the `VariantDesc` the
   picker held under that name equals the entry's `menu`, `axes` and
   `label_template`. It did not compile until the fields existed and passed
   once the emit filled them, while the constructor was still there.
2. Delete the self registration features, their registration modules and the
   dependencies only they needed, one family crate at a time.
3. Add the fields and fill them in all seven emit arms. The `spec_emit` tests
   that asserted on the registration call assert on the fields instead.
4. Add `array_vis_bench_core::navigation` and point the CLI at it. The CLI
   tests that build a `VariantDesc` by hand keep working unchanged.
5. Point `sorted` and `validate_registries` at the entries, and the catalog
   tests at `variants()`.
6. Delete the constructor from the emit, with `AbiPaths.nav`, and the bridge
   test with its second source. Then delete the crate.

## Checks

- `catalog.golden` and the entry fingerprint are unchanged throughout.
- No `#[ctor]` remains in `catalog_entries.rs`; the two constructors in
  `bench_registry` (validation and subprocess dispatch) are the only ones
  left.
- A tree wide search for the crate name, its registration functions and the
  feature name finds nothing.
- The picker view lists exactly the 1580 entries and every entry has a label
  template.
- Time `cargo run --release -- --timing 1 1` before and after; the
  constructor storm was the expected saving.
