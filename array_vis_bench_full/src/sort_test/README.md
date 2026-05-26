# sort_test

Runtime test harness used outside `cargo test`. Looks algorithms up by name in `bench_registry::ALGORITHMS` and runs their category-appropriate correctness battery via `entry.run_correctness`.

## Functions

- `test_sort(choice)` — find the algorithm named by `choice` in `bench_registry::ALGORITHMS` and run its correctness battery. Returns `true` on success.
- `test_all()` — iterate every registered algorithm and run each one's battery.

## Relationship to `#[test]` tests

The authoritative correctness infrastructure lives in `bench_registry::correctness` (called from per-family generated tests and the `all_registered_algorithms_are_correct` aggregate `#[test]` in `bench_registry.rs`). Those `#[test]` cases run each algorithm in a subprocess with a wall-clock timeout, capture `RUNNING:` breadcrumbs on TLE, and panic with a per-algorithm failure summary.

This module is a lighter runtime alternative — useful when iterating in the visualiser or speed-test binary where you want to validate one algorithm without spinning up the full subprocess battery.
