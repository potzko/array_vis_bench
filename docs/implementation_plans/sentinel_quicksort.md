# Implementation plan: SentinelQuickSort (sentinel-based bounds-check elimination)

Status: **DESIGNED, NOT STARTED (2026-06-30).** Keep as a parked perf lever; do not implement until
a benchmark justifies it (see "When to build" below). The pivot-under-partition re-model it builds on
is fully DONE (2026-07-01): the catalog emits every pivot-based quicksort as
`CanonicalQuickSort<Pivoted<Scheme, Sel>, SS>` directly, the pivot NESTS under the partition in names
(`quick sort<part: left-left<first>, small: …>`), the top-level `project pivot` coupling is gone
(arity baked into single/dual `Pivoted` composites), and quick shares ONE partition role-system with
the first-class Partition kind. So the only tail left for THIS plan is surfacing sentinel-capable
schemes (a `sentinel` tag) — see phase 5.

## The optimisation

A pivot-based partition's inner scans carry a loop-bound guard that a **sentinel** makes redundant.
Hoare (`LeftRightPartition`, [partition_hoare/src/lib.rs](../../crates/partitions/partition_hoare/src/lib.rs))
is the clearest case — under `NoOpLogger` its two scans lower to:

```rust
while left <= right && arr[left] <= pivot { left += 1; }   // left scan
while left <= right && arr[right] > pivot { right -= 1; }   // right scan
```

- the **left** scan stops on its own if a known element **> pivot** sits to its right → its
  `left <= right` guard is dead iff a **right sentinel** exists;
- the **right** scan stops if a known element **≤ pivot** sits to its left → its guard is dead iff a
  **left sentinel** exists.

Two independent bits → **4 partition variants** (neither / left-only / right-only / both), each with
the corresponding inner guard compiled out. Hand-rolled optimised quicksorts (Sedgewick,
Bentley–McIlroy) use exactly this to tighten the hot loop; closing the last few % to hand-rolled is
a stated project goal.

## Why this needs a *driver*, not just a partition

The sentinel bits come from **recursion-tree position**, which the partition (seeing one slice)
cannot know — only the driver, which placed the pivots, can:

- partition `[lo,hi]`, pivot lands at `j` → the **left child `[lo,j)`** has `arr[j]` (≥ all of it) as
  a **right sentinel**; the **right child `(j,hi]`** has `arr[j]` (≤ all of it) as a **left
  sentinel**; each **inherits** the parent's *outer* sentinel.
- the **root** call has neither (both ends are the array bounds).

So the driver threads a 2-bit `BoundCtx { left_sentinel, right_sentinel }`, derives each child's ctx
from its position + inherited outer ctx, and dispatches to the matching guard-free partition. The
canonical driver's bare "unsorted ranges" visitor contract deliberately discards this — which is why
this is a genuinely different driver, not a relabel.

## Architecture

A **second** quicksort driver, parallel to the canonical one:

- `CanonicalQuickSort<P, SS>` (the existing bare driver, formerly `PivotlessQuickSort`): contract =
  self-splitting `PartitionScheme` (`N_PIVOTS == 0`); visitor only; pivot-agnostic. **Stays the
  default** for every scheme (HeapExtract, plain `Pivoted<Scheme, Sel>`, anything not sentinel-aware).
- `SentinelQuickSort<P, SS>` (**new**): threads `BoundCtx`, runtime 4-way dispatch per call into a
  **bound-aware** partition entry. **Opt-in** for the pivot schemes that implement that entry.

These are two real algorithms (one carries ctx + dispatches, one doesn't) — *not* the four
near-identical drivers the migration just collapsed.

### Partition contract extension (const-generic, so the hot loop is guard-free)

Add a bound-aware entry to `PartitionScheme` (or a sibling trait `SentinelPartition`):

```rust
fn partition_bounded<const L_SENTINEL: bool, const R_SENTINEL: bool, T, U, V>(
    arr, logger, pivots, scratch, visitor,
);
```

- The const generics make the guards **statically present/absent** after monomorphisation — no
  runtime branch *inside* the scan. The driver does ONE runtime `match ctx { (F,F) => …::<false,false>, … }`
  per partition call (a predictable, hoistable branch), then the inner loop is clean.
- **Default impl** = today's fully-guarded body (ignores the consts), so every existing scheme gets
  it for free and only schemes that genuinely benefit (Hoare first) override with the four guard-elided
  forms. `BoundCtx` cannot thread as a const through recursion (depth is runtime), hence runtime
  ctx + const-generic dispatch.

### Where the pieces live

- **Driver**: `BoundCtx` bookkeeping + the 4-way dispatch. Computes children's ctx from the visitor's
  range geometry (a child range abutting a placed-pivot gap gains a sentinel on that side) + inherited
  outer ctx.
- **Partition**: the four guard-elided inner-loop bodies behind the const-generic entry.
- **Pivot selection**: unchanged — still inside the partition (`Pivoted<Scheme, Sel>` or a fused
  bespoke scheme). Only the *bound context* moved to the driver, not the pivot value.

## Phasing

1. **`partition_bounded::<L,R>` entry** on `PartitionScheme` with a default = the current body. No
   behaviour change; everything still routes through `CanonicalQuickSort`. (Behind existing tests.)
2. **Guard-elided Hoare.** Implement the four variants for `LeftRightPartition`; unit-test each
   against the guarded version on the same inputs (incl. heavy-duplicate arrays — see correctness
   caveat).
3. **`SentinelQuickSort<P, SS>` driver.** `BoundCtx` threading + child-ctx derivation + 4-way
   dispatch. A scheme without the override just falls back to the guarded body, so this is safe to
   point at any scheme.
4. **Benchmark gate.** `compare_sorts` (NoOpLogger): does `SentinelQuickSort<Pivoted<Hoare, …>>` close
   the gap to the hand-rolled baseline that `CanonicalQuickSort` leaves open? If the canonical path is
   already inside the 1.05× gate, **stop here — it's YAGNI.**
5. **Catalog / spec wiring (deferred, overlaps structural-nav).** Surface sentinel-capable schemes as
   a quick-sort sub-family or a strategy axis. Names shift (a `sentinel` tag), so it lands with the
   structural-nav picker/name reshape (the same tail that would nest pivot under partition), not before.

## Honest caveats

- **Only pays on the `NoOpLogger` / perf path.** The visualiser logs every comparison through
  `SortLogger` regardless, so guard-elision is invisible there — this is purely a hand-rolled-parity
  lever, measured by `compare_sorts`, not a visualisation feature.
- **Equal keys are the correctness trap.** A sentinel stops a `<= pivot` scan only if it is *strictly*
  `> pivot`; an *equal* neighbour lets the scan run past it (→ out-of-bounds / corruption). Real
  sentinel quicksorts dodge this via median-of-3 / strict placement. This is precisely what the
  transitivity-safety correctness work (the sort battery in `array_vis_bench_core/src/bench_registry.rs`) must cover before
  this ships — guard elision is where an order-relation bug stops being caught by "is it sorted?" and
  starts being UB.
- **Cost vs win.** Adds a second driver + four partition bodies per opted-in scheme. Justified only by
  the phase-4 benchmark. Default stays `CanonicalQuickSort` + `Pivoted`.

## When to build

When `compare_sorts` shows `CanonicalQuickSort<Pivoted<Hoare, …>>` trailing the hand-rolled Hoare
baseline by more than the ~1.05× gate on the partition-bound path. Until then: parked.

## Pointers

- Drivers: [crates/families/quick_sort_lib/src/quick_sort.rs](../../crates/families/quick_sort_lib/src/quick_sort.rs)
  (`CanonicalQuickSort`), [deferred_quick_sort.rs](../../crates/families/quick_sort_lib/src/deferred_quick_sort.rs).
- Partition contract + visitor: [array_vis_bench_traits/src/role/partition.rs](../../array_vis_bench_traits/src/role/partition.rs).
- The poster-child scheme: [crates/partitions/partition_hoare/src/lib.rs](../../crates/partitions/partition_hoare/src/lib.rs).
- Perf gate: `src/bin/compare_sorts.rs`. Correctness: the sort battery in
  `array_vis_bench_core/src/bench_registry.rs` (transitivity safety is open work, see
  [../todo.md](../todo.md)).
