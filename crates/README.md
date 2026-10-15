# crates

The standard library the catalog composes from. Every crate here depends on
`array_vis_bench_traits` and `sort_logger` and nothing heavier, declares its
components in a `.spec` fragment, and is gathered by `spec_catalog`.

- `families/`: drivers and composers, one crate per sort family.
- `partitions/`: one crate per partition scheme.
- `pivots/`: one crate per pivot selector.
- `rotations/`: one crate per rotation algorithm.
- `small_sorts/`: small sort strategies and sorting networks.
- `components/`: shared components that are not an algorithm on their own.
