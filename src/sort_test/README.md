# sort_test

Runtime testing harness for validating sort correctness. Looks up sorts by name in `BENCH_SORTS` and runs them against test arrays.

## Functions

- `test_sort(choice)` — find the sort named by `choice` in `BENCH_SORTS` and run it against a set of test arrays. Returns `true` if all checks pass.
- `test_all()` — run the test suite against every sort registered in `BENCH_SORTS`.

## Relationship to `#[test]` tests

The primary sort test infrastructure lives in `bench_registry::test_helpers::check_sort`, which is called automatically by every sort registered via `create_sort!` or `sort_family!`. Those `#[test]` tests are more comprehensive (exhaustive small permutations, random arrays up to 5000, duplicate patterns, stability checks). This module is a lighter runtime alternative for use outside `cargo test`.
