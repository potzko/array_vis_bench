# `sort_registry_core`

The picker's navigation registry. Each emitted entry's constructor calls
`register_sort_variant` with a category path and its `(role, value)` facet
axes; the interactive CLI reads `all_variants` and navigates the categories
as a tree and the axes one role at a time. No function pointers live here;
those are in `array_vis_bench_core::bench_registry::ALGORITHMS`.

- `src/lib.rs`: `VariantDesc`, `AxisBinding`, the registration entry points and the tree builders.

It is a tiny leaf with no project dependencies so both the emit and the CLI
can reach it without depending on each other.
