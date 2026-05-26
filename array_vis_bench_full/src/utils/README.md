# utils

Shared algorithmic building blocks used across sort families. Nothing here is a sort — these are the reusable primitives that sorts compose.

## Subfolders

### `rotation/`
Array rotation algorithms — 11 implementations of the operation "move `arr[split..]` to the front and `arr[..split]` to the back". Used by rotation merge sorts to merge in-place without auxiliary memory. See [rotation/README.md](rotation/README.md).

### `shell_sequences/`
Gap sequence generators for shell sort. Each sequence produces a descending list of gap values that determine the stride pattern. See [shell_sequences/README.md](shell_sequences/README.md).

### `shell_branching/`
Branching strategies for shell-shell sort and rod sort. Control how many interleaved sub-arrays each recursive level splits into. See [shell_branching/README.md](shell_branching/README.md).

## Files

- `array_gen.rs` — array generators for testing and benchmarking: `get_rand_arr`, `get_arr` (ascending), `get_reversed_arr` (descending), `get_rand_arr_in_range`.
- `check_utils.rs` — post-sort verification: `is_sorted` (order check) and `is_sorted_arr` (permutation + order check, verifies no elements were lost or duplicated).
- `mod.rs` — re-exports all submodules and provides `read_num_stdin()` for the interactive CLI.
