# `src`

Source of `array_vis_bench_traits`. The crate README one level up says what it is for.

- `complexity.rs`: Compile-time complexity annotations.
- `composable.rs`: Composable per-axis annotations for sort algorithms.
- `lib.rs`: Pure-trait crate for the `array_vis_bench` workspace.
- `sort_traits.rs`: Indirect-dispatch sort trait: routes through `<T as SortAlgo<...>>::sort(arr, logger)` instead of an inherent method.
- `role/`: see its README.
