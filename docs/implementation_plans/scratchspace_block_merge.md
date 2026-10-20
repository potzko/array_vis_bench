# Implementation plan: `Auxiliary` (scratchspace) role + block-merge sorts

Status: **SHIPPED into the catalog (2026-07-02).** The scratchspace concept now fits the existing
role/merge structure and rides the real AVBS pipeline — no standalone scaffolding:
- **`Auxiliary` role trait** (`array_vis_bench_traits/src/role/auxiliary.rs`): `NAME` + `scratch_size(n)`
  + complexity consts, mirroring `Rotation`'s "small auxiliary" convention (scratch allocated via the
  logger, threaded in; `0` = in-place). This is the composable axis AVBS slots over.
- **`scratchspace_lib`**: three backings — `FullBuffer` (`O(n)`), `HalfBuffer` (`O(n/2)`),
  `NoBuffer` (`O(1)`) — plus `AuxiliaryMerge<A>`, which turns any `Auxiliary` into a `RotationMerge`.
  **Every element move goes through the logger** (`copy_range`/`write`/`write_accross` buffered,
  `swap` in-place); no `mem::swap`, nothing outside the logger.
- **Registered as `Category::Merge` kinds** via `scratchspace.spec` (`aux_full|half|none` provide the
  `Auxiliary` role; `mk_aux = AuxiliaryMerge<{aux}>` provides `MergeKind`). `spec_catalog`'s
  `Merges: MergeAlg<_>` query enumerates them automatically → `auxiliary<full|half|none>` under the
  `merges` picker. The existing **Visualiser / Benchmark / Correctness** mains drive them unchanged
  (`each_new_kind_ticks_through_all_three_mains` + `merge::all_emitted_entries_pass_correctness` pass;
  6 crate unit tests green; clippy-clean).

**Top-level `Scratchspace` kind with a full operation contract (2026-07-02).** A *separate* first-class
kind (its own menu item, not folded into `merges`): the `Scratchspace` role trait
(`array_vis_bench_traits/src/role/scratchspace.rs`) + `Category::Scratchspace`.

The trait is a real ADT, not a single `roll`. Operations (each takes an `aux` buffer, rotation-trick
style — `aux_size(n)` declares how much to allocate, `0` = in-place):
- lifecycle: `acquire` / `release` (default no-op hooks);
- borrow pair: `park(list, at, len, aux)` / `place(list, at, len, aux)`;
- movement: `block_swap`, `relocate(from, to, len)` (parks the block when it fits, else rotates),
  `roll(lo, hi, k)` (three-reversal rotate);
- `exercise` — a guided round-trip running acquire→park/place→block_swap→relocate→roll→release, each
  paired with its inverse so the **net effect is identity** while every op is driven through the logger.
- Invariant all ops uphold: **conservation** (multiset of `list ∪ aux` preserved).

`scratchspace_lib` provides three **backings** (the `Scratchspace` role) realising the ops with
different aux budgets: `FullScratch` (`O(n)` → all buffered), `SqrtScratch` (`O(√n)` → small runs
park, big ones rotate), `NoScratch` (`O(1)` → `park` unavailable, `relocate`/`roll` run by rotation).

**Workflows are a second axis** (`ScratchspaceRun` role): a workflow is a scripted routine over the
ops, cross-producted with the backing. Four ship — `exercise` (guided tour of every op), `park-sweep`
(park 100 windows without placing), `park-loop` (park→place ×20), `relocate` (move a block to
near-start / ¼ / ¾ and back, like the rotation kind's targets). Each nets to **identity**, so the
whole `scratchspace_battery` verifies the contract round-trips. Emitted as
`<workflow><backing>` (12 variants) under the `scratchspaces` picker.

**The nightmare sort** (`ScratchspaceMergeSort<S>`, `scratchspace-nightmare<full|sqrt|none>` under
fun sorts / `Category::Sort`): a real stable natural-merge sort built entirely on the scratchspace —
detect two adjacent ascending runs, park the left, merge, repeat pass-by-pass until one run spans the
array; buffered merges when the aux fits, `O(n²)` in-place merge when it doesn't (so the zero-aux
backing is a genuine nightmare). Correctness = `sort_battery` (own `scratchspace.avbs` query).

The **whole contract** is what the consumers drive: `run_scratchspace` allocates the aux via the logger
and runs `exercise`, so Correctness (`ScratchspaceSuite` → `scratchspace_battery`, "list unchanged"),
Visualiser (buffer tile fills/drains on park/place, blocks shift on relocate, range rotates on roll),
and Benchmark all cover park/place/relocate/block_swap/roll — not just a roll. Wiring: `Category` +
`as_str` + `list_inputs`/`primary_input` (reuse `SORT_INPUTS`) + `run_scratchspace_with_input` +
`scratchspace_battery` + `ScratchspaceSuite` + `spec_emit::scratchspace_entry_body` +
`ScratchspaceAlg`->`ScratchspaceKind` in `spec_core::avbs` + `kinds.avbs`. Ticks through
all three mains (67 catalog tests green; 13 leaf tests incl. per-op contract checks; clippy-clean).

**Pivot note:** the earlier standalone design (a `Scratchspace<T>` trait with `InArray`/`ExternalFixed`/
`ExternalDyn`/`NoScratch` backings + `bottom_up_merge_sort` + `examples/` rendering an mp4) was
removed in favour of fitting the repo's role system. The backing *idea* survives as the three
`Auxiliary` impls; the richer in-array **swap-based** buffer (WikiSort/GrailSort, where writes displace
real elements and must be restored) is the phase-4 direction below, not what ships today.

**Known ceiling:** `NoBuffer` (and any backing once a run outgrows its buffer) uses an `O(n²)` in-place
insertion merge. `O(n log n)` needs block-rolling — phase 4.

Design rationale & the research this rests on: the WikiSort/GrailSort/stable-quicksort deep-research
report + the "types-as-composition scratchspace contract" reflection.

## What this is

WikiSort and GrailSort are both **block merge sort**: a stable, in-place `O(n log n)` merge sort
that does its merges using a small **internal buffer** carved from (or allocated beside) the array
instead of an `O(n)` scratch array. The buffers across the whole family differ on exactly **one
axis** — *backing*:

| Backing | Where the workspace lives | Writing to it… | Space |
|---|---|---|---|
| **in-array** (WikiSort, canonical GrailSort) | a region of the array itself, filled with borrowed real elements | is a **swap** (must be restored) | `O(1)` |
| **external-fixed** (GrailSort `WithBuffer`, 512) | a stack array | is a free **overwrite** | `O(1)`, bigger constant |
| **external-dyn** (GrailSort `WithDynBuffer`, √n) | a heap `Vec` | a free overwrite | `O(√n)` |
| **none** (bufferless / SymMerge fallback) | nothing | n/a — merge by rotation | `O(1)` |

Everything else people call "the buffer" is either the same substrate under a different
*interpretation* (the key/tag buffer is a permutation-encoded label directory — a Movement-Imitation
Buffer) or a genuinely different beast (the stable-quicksort partition's `O(log n)` counting space —
**explicitly out of scope**, it shares nothing but the logger verbs).

So the contract is small, and *backing* is the composition type — matching the repo's
types-as-composition style (cf. `CanonicalQuickSort<Pivoted<Scheme, Sel>, SS>`).

## The contract (as shipped)

`Auxiliary` is a **role trait** — the scratchspace axis a merge composes over. It mirrors `Rotation`:
declare how much scratch you need; the pipeline allocates it via the logger and threads it in.

```rust
// array_vis_bench_traits::role::Auxiliary
pub trait Auxiliary {
    const NAME: &'static str;          // variant label: auxiliary<full|half|none>
    const MERGE_WORST: Complexity;     // O(n) buffered, O(n²) in-place
    const MERGE_SPACE: Complexity;
    fn scratch_size(n: usize) -> usize; // 0 = in-place
}
```

`scratchspace_lib::AuxiliaryMerge<A: Auxiliary>` consumes one and **is** a `RotationMerge`
(`scratch_size` delegates to `A`; `merge` runs a logged half-copy when the buffer holds the left run,
else a logged in-place insertion merge) plus `HasTimeBounds`/`HasSpace`/`HasStability` (inherited from
`A`). So it drops straight into the `Category::Merge` emit (`run_rotation_merge::<Ty>`).

**The law (asserted by the crate tests):** merging two sorted halves yields the full multiset, sorted
and **stable** (equal keys keep original relative order), for every backing. All movement is via the
logger — nothing else touches the array.

**Backings as the composition axis:** `FullBuffer` (`O(n)`), `HalfBuffer` (`O(n/2)`), `NoBuffer`
(`O(1)`). `NoBuffer` is a first-class member (bufferless insertion merge), not an error path.

## Deliberately out of scope

- **Swap-based in-array buffer (true WikiSort/GrailSort).** Today's backings are the *external-size*
  axis (how big an aux buffer). The in-array buffer whose writes **displace** real elements (and must
  be restored) needs a cross-array swap the logger doesn't expose yet — that's phase 4/5, alongside
  block-rolling and a "buffer region" logger event.
- **Keys / tags (Movement-Imitation Buffer).** A sibling axis (label directory, restored by
  insertion-sorting the scramble), not an `Auxiliary` method.
- **Stable-quicksort partition space.** Different animal (`O(log n)` counting/bits). Not part of this.

## Phasing

1. **`scratchspace_lib` component crate (DONE).** The trait + `InArray` + `ExternalFixed<512>` + a
   `buffered_merge`/`merge_runs` core that consumes any `Scratchspace` + self-check tests asserting
   the law on both backings (incl. heavy-duplicate/already-sorted inputs and the `capacity < want`
   degraded path). Standalone; no `ALGORITHMS` wiring.
2. **Round out backings + a real consumer (DONE).** Added `ExternalDyn` (runtime √n) and `NoScratch`
   (capacity-0, bufferless) — the same law/sort battery runs over all four. Added
   `bottom_up_merge_sort<S>`, so the trait composes into an actual stable sort. *Deferred within this
   phase:* the bufferless path is insertion merge, not yet the repo's rotation components
   (`crates/rotations/*`) — swap that in when `NoScratch` needs to be sub-quadratic (folds into
   phase 4's block-rolling).
3. **The `Keys` sibling axis.** A `KeyBuffer` type (extract distinct elements, `tag_block` /
   `find_block`, `restore` = insertion-sort). Same acquire/release *shape*, its own methods.
4. **`block_merge_sort_lib` family.** `BlockMergeSort<Merge, Keys>` implementing `SortAlgo` +
   `Has{TimeBounds,Space,Stability}`. WikiSort ≈ `<InArray, InArrayKeys>`; GrailSort-with-buffer ≈
   `<ExternalFixed<512>, InArrayKeys>`. Buffer-extraction failure (too few distinct elements) falls
   back down the backing ladder — log what was dropped (no silent cap).
5. **Registration + visualisation (deferred).** Emit as `AlgorithmEntry`s via the catalog/compiler
   path; add the one genuinely-new logger event this needs — *"this sub-range is currently the
   buffer / the key region"* — so the visualiser can shade the buffer while blocks roll and tags
   swap. Until a sort needs the highlight, don't build the event.

## Validate the contract against these (before committing to phase 4)

Each stress-tests a different corner; if any won't fit, the contract is wrong:

- **`std::inplace_merge` / libstdc++ `__merge_adaptive`** — grabs a temp buffer, **falls back to a
  bufferless rotation merge when allocation is unavailable or partial**. Validates the whole backing
  spectrum + graceful degradation (⇒ `acquire` yields ≤ `want`, never panics).
- **Bufferless rotation merge (SymMerge / Kim–Kutzner symmetric)** — the **zero-capacity** case (⇒
  `acquire` returns `Self`, never `Option`; `NoScratch` must be legal).
- **Timsort `merge_lo`/`merge_hi` temp** — external temp sized to the *smaller run*, per-merge (⇒
  size is a per-`acquire` runtime `want`, not a type-fixed const).
- **Movement-Imitation Buffer (Mannila–Ukkonen) / GrailSort keys** — permutation-encoded info, not
  values (⇒ keeps keys as a separate axis, not a `Scratchspace` method).

## Honest caveats

- **"in-place" is word-RAM `O(1)`** (a constant number of index words), the field's standard meaning
  — the in-array backing carries no heap, but does move real elements around via swaps.
- **Buffer extraction can starve.** With too few distinct elements there aren't enough unique values
  to form the buffer (and keys). The fallback is down the backing ladder to `NoScratch`; it must be
  a logged, first-class path, not a panic — mirror the "no silent caps" discipline.
- **The visualiser only needs phase 5.** The trait + merges are correctness-visible under any
  logger; the buffer-region shading is the only new consumer-side surface, and it's deferred.

## Pointers

- New home: `crates/components/scratchspace_lib/` (sits beside `reverse_storage_lib`,
  `gap_distribution_lib`).
- Merge consumers land in `crates/families/merge_sort_lib/` (already has `rotation_merge.rs`,
  `auxiliary_merge.rs`) or a new `crates/families/block_merge_sort_lib/`.
- ABI: `SortLogger` (`sort_logger`), `SortAlgo` + `Has*` traits (`array_vis_bench_traits`).
- Research: the deep-research report (WikiSort README + Ch.3, Mrrl/GrailSort.h, Huang–Langston 1992,
  Kim–Kutzner 2008). Rotation building blocks already in-repo: `crates/rotations/*`.
