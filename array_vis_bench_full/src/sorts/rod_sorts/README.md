# rod_sorts

Rod sort -- a recursive divide-and-conquer sort that works on interleaved virtual sub-arrays, parameterised over branching strategies.

## How rod sort works

At each recursion level, rod sort splits the array into `branch` interleaved sub-arrays at increasing stride, recursively sorts each, then merges with an insertion-sort pass. The name "rod" comes from the visual metaphor of interleaved rods being sorted independently, then combined.

## Branching strategies

Rod sort is generic over `S: BranchingStrategy`. See [shell_branching/README.md](../../utils/shell_branching/README.md) for the available strategies (Classic, Parity3, LogParity, RootParity, Optimised, Fibonacci).

## Files

- `rod_sort.rs` -- `RodSort<S: BranchingStrategy>` implementation.
- `branching.rs` -- `ROD_STRATEGIES` distributed slice with per-strategy registration entries.
- `combinations.rs` -- generates and registers all variant combinations.

## Status

Active — wired into the dispatch tree in `sorts/mod.rs`. Uses distributed-slice registration.
