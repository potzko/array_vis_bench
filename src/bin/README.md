# bin

Additional binary targets beyond the default `array_vis_bench` visualiser.

## `speed_test.rs`

A lightweight speed comparison tool that bypasses Criterion overhead. Runs every sort registered in `BENCH_SORTS` against a shuffled array, measures raw wall-clock time (median of N runs), and prints results sorted fastest-first.

Usage: `cargo run --bin speed_test [array_size] [num_runs]` (defaults: 100,000 elements, 7 runs).

Useful for quick iteration when you want a rough performance ranking without waiting for Criterion's statistical analysis.
