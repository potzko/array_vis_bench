# sort_test

Testing harness for validating sort correctness. Runs a sort against multiple test arrays and verifies both ordering and element preservation.

## What it checks

- The output is sorted (monotonically non-decreasing).
- The output is a permutation of the input (no elements lost or duplicated).

## Files

- `general_test.rs` — `test_sort(choice)` runs the sort identified by `choice` against a set of test arrays (random, ascending, descending, all-equal, etc.) and returns `true` if all checks pass.
- `mod.rs` — re-exports `test_sort`.
