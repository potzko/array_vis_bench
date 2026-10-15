# `array_vis_bench_min`

A minimal link root: two families and their direct dependencies, no catalog,
no picker, no renderer. It demonstrates that a consumer can take just the
leaves it wants. Run with `cargo run -p array_vis_bench_min --release`.

- `src/main.rs`: heap sort and insertion sort on a random array.
- `src/main_with_quick.rs`: the same plus one quick sort with one partition and one pivot.
