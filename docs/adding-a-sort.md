# Adding an algorithm

The full guide, with a worked example, is [writing-components.md](writing-components.md).
This page is the short recipe for the three common cases.

## A new slot filler (pivot, partition, rotation, small sort)

1. Make a crate under `crates/<axis>/<name>/` depending only on
   `array_vis_bench_traits` and `sort_logger`.
2. Implement the role trait, routing every comparison and write through the
   logger, plus `HasTimeBounds`, `HasSpace`, `HasStability` and, for a pivot,
   `PivotQuality`.
3. Write `<name>.spec` with one component: `type`, `label`, `provides <Role>`.
   The type must be a `pub` item at the crate root, since the build imports it
   from there.
4. Point the manifest at it with `[package.metadata.array_vis_bench] spec = "<name>.spec"`.
5. Add the crate to the workspace members and to `spec_catalog/Cargo.toml`.
6. `cargo build`, then `cargo test -p spec_catalog`. The golden counts will
   move; update them to the new numbers once you have checked that the new
   variants are the ones you expect.

A filler needs no query. Every driver whose slot has that role picks it up.
If some consumer should not see it, subtract it in that consumer's query.

## A new family with its own driver

1. Make a crate under `crates/families/<name>/` with the driver type, generic
   over its slots, and an inherent `sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>`.
   Join the `DirectSort` role with one line beside the impl,
   `array_vis_bench_traits::direct_sort!(Driver<S>; S: Role);`, repeating the
   impl's generics after the `;`.
2. Give it the four annotation impls, composed from its parts with
   `Complexity::sum` and `Complexity::product`.
3. In `<name>.spec` declare the driver component with a `slot` per type
   parameter, a `menu` path, and any family local fillers.
4. In `<name>.avbs` write the query, usually
   `Name: SortingAlgorithm = SortingAlgorithm<Driver<slot = _, other = _>>`.
   Curate with `_ - {...}` or `where (...)` rather than branching in the
   driver.
5. Declare both files in the manifest, add the crate to the workspace and to
   `spec_catalog/Cargo.toml`.
6. Add a golden count assertion in `spec_catalog/src/lib.rs` beside the other
   families.

If a slot role can be filled by something that contains the same role, wrap
the hole in `finite(_)` or add a `depth N` directive, or the build will refuse
the query as unbounded.

## A new first class kind

A kind is a new `Category` with its own wrapper, role and suite. The four
places to touch are listed at the end of [registration.md](registration.md):
a `Category` variant, a row of `spec_emit`'s `KINDS` table, a `<Kind>Suite`
impl and the input tags. The scratchspace kind is the reference
implementation.

## Rules every implementation follows

- Every comparison and every write goes through the logger. Reading into a
  temporary is fine. A raw `arr.swap` works in the benchmark and silently
  desynchronises the visualiser.
- Prefer the fused helpers (`cond_swap_lt`, `cond_swap_gt`, `swap`) over
  hand rolled compare and write pairs.
- Pass the same slice object consistently. The logger identifies an array by
  its base pointer, so a reslice looks like a different array.
- Auxiliary storage that grows with the input is allocated and freed through
  the logger so the renderer can show it.
- A sort that cannot handle large random inputs declares `max_input` in its
  spec. A sort whose trace is intentionally random declares
  `nondeterministic true`.
- Nested loop indices are `i`, `ii`, `iii`.
