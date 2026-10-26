# `array_vis_bench_traits`

The pure trait crate every leaf implements against. It depends only on
`sort_logger`.

- `src/role/`: the role traits, one file per axis (partition and its visitor, pivot, small sort, rotation, merge, direct sort, quick select, auxiliary, scratchspace).
- `src/composable.rs`: `HasTimeBounds`, `HasSpace`, `HasStability`, `PivotQuality`.
- `src/complexity.rs`: the const evaluable Big-O class with `sum` and `product`.
