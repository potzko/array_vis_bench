# shell_branching

Branching strategies for shell-shell sort and rod sort.

## What is a branching strategy?

Shell-shell sort and rod sort are recursive sorts that work on *virtual sub-arrays* — elements spaced `stride` apart. At each recursion level, the algorithm splits the virtual array into `branch` interleaved sub-arrays (each at `stride × branch` spacing), recursively sorts each one, then merges with an insertion-sort pass at the current stride.

The branching strategy controls three things:

1. **`branch(virtual_len)`** — how many sub-arrays to split into. Higher values mean more recursion but smaller sub-problems.
2. **`should_cut(virtual_len)`** — when to stop recursing and switch to direct insertion sort. The base case.
3. **`intermediate(virtual_len)`** — optional extra insertion-sort pass after recursion, at a coarser stride, to smooth out partial disorder before the final merge. Return 0 to skip.

## The `BranchingStrategy` trait

```rust
pub trait BranchingStrategy {
    const NAME: &'static str;
    const BIG_O: &'static str;
    fn should_cut(virtual_len: usize) -> bool;
    fn branch(virtual_len: usize) -> usize;
    fn intermediate(virtual_len: usize) -> usize { 0 }
}
```

## Strategies

| Type | Branch factor | Behaviour |
|---|---|---|
| `Classic` | 2 (constant) | Binary split, never cuts early. The simplest strategy — equivalent to a fixed gap sequence of powers of 2. |
| `Parity3` | 3 (constant) | Ternary split. Cuts when `virtual_len < 2`. |
| `LogParity` | floor(log2(len)) | Adaptive: larger arrays split more aggressively. Cuts at `< 16`. Includes an intermediate pass at `branch - 1`. |
| `RootParity` | floor(sqrt(len)) | Even more adaptive. Cuts at `<= 4`. Includes an intermediate pass. |
| `Optimised` | 32 (constant) | Large fixed branching with an intermediate pass at 15. Cuts at `< 64`. Tuned for good practical performance. |
| `Fibonacci` | fib-index nearest to len | Fibonacci-derived branching factor. Cuts at `< 16`. Includes an intermediate pass. |
