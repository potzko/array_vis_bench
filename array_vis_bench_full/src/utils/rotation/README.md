# rotation

Array rotation algorithms — 11 implementations of the same operation with different performance trade-offs.

## What is a rotation?

A rotation (also called a block swap or cyclic shift) takes a slice and a split index, and rearranges so that `arr[split..]` comes first and `arr[..split]` moves to the back:

```
Before: [A A A | B B B B B]
              ^split
After:  [B B B B B | A A A]
```

Equivalent to `arr.rotate_left(split)` in std, but instrumented through `SortLogger` so every data movement is recorded for visualisation.

## Why rotations matter

Rotation merge sorts merge two sorted halves in-place by rotating elements into their correct position — no auxiliary array needed. The choice of rotation algorithm directly affects the merge sort's constant factors, cache behaviour, and auxiliary memory usage. Having 11 implementations lets us benchmark and visualise how each one affects the overall sort.

## The `Rotation` trait

```rust
pub trait Rotation {
    const NAME: &'static str;
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T], split_ind: usize, logger: &mut U,
    );
}
```

All algorithms implement this trait with the same semantics. Sort code is generic over `R: Rotation`, so swapping the rotation algorithm is a type-parameter change.

## Algorithms

### In-place, no auxiliary memory

| File | Type | Method | Year |
|---|---|---|---|
| `reversal.rs` | `ReversalRotation` | Triple reversal: reverse left, reverse right, reverse all. Simple and cache-friendly but touches every element 2x. | Pre-1981 |
| `gries_mills.rs` | `GriesMillsRotation` | Repeated block swaps, shrinking the remainder each iteration. O(1) extra space, good locality. | 1981 |
| `juggling.rs` | `JugglingRotation` | GCD cycle-based: follows `gcd(n, split)` independent cycles, each element moved exactly once. Optimal moves but poor cache locality on large arrays. | 1965 |
| `contrev.rs` | `ContrevRotation` | Conjoined triple reversal — fuses the three reversal phases into interleaved pointer walks, reducing passes over the data. | 2021 |
| `piston.rs` | `PistonRotation` | Successive forward block swaps, alternating which side is the remainder. Similar to Gries-Mills but always swaps forward. | 2021 |
| `helix.rs` | `HelixRotation` | Grail-derived with alternating inner loops. Falls back to a small buffer for the base case. | 2021 |
| `drill.rs` | `DrillRotation` | Combines grail, piston, and helix inner loops. Falls back to a small buffer for the base case. | 2021 |

### With auxiliary memory

| File | Type | Method | Year |
|---|---|---|---|
| `auxiliary.rs` | `AuxiliaryRotation` | Copy the smaller side to a temporary buffer, shift the larger side, copy back. O(min(left, right)) extra space. Simplest and often fastest for small rotations. | 2021 |
| `bridge.rs` | `BridgeRotation` | Minimizes aux to `|left - right|` elements (the "bridge"). When the bridge is small, uses a buffer + pointer walk; otherwise falls back to auxiliary rotation. | 2021 |
| `grail.rs` | `GrailRotation` | Gries-Mills until the remainder is 1 element, then finishes with a small auxiliary rotation. Balances in-place efficiency with a fast tail. | 2020 |
| `trinity.rs` | `TrinityRotation` | Hybrid: uses up to 8-element auxiliary buffer for small sides/bridges, contrev rotation for larger cases. Designed to be fast across all input sizes. | 2021 |

## Shared helpers (in `mod.rs`)

- `reverse(arr, logger)` — in-place reversal.
- `forward_block_swap` / `backward_block_swap` — swap N elements between two positions.
- `buf_rotate_left` / `buf_rotate_right` — auxiliary-buffer rotation in a given direction.
- `gcd(a, b)` — Euclidean GCD, used by juggling rotation.

## Attribution

Ported from [scandum/rotate](https://github.com/scandum/rotate) (MIT, Igor van den Hoven).
