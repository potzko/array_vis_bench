# Implementation plan: `Auxiliary` (scratchspace) role + block-merge sorts

Status: **SHIPPED into the catalog (2026-07-02), phases 3 to 5 and the bufferless merge shipped 2026-10-28, see [As built](#as-built-the-keys-axis-the-block-merge-sort-the-region-event).** The scratchspace concept now fits the existing
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
array; buffered merges when the aux fits, the bufferless rotation merge when it doesn't (so the zero-aux
backing pays `O(n log n)` moves for every merge). Correctness = `sort_battery` (own `scratchspace.avbs` query).

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

**The ceiling is gone (2026-10-28):** the bufferless backings, and any backing once a run outgrows its buffer, merge by rotations in `O(n log n)` moves, not by an `O(n^2)` insertion merge.

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
    const MERGE_WORST: Complexity;     // O(n) buffered, O(n log n) in-place
    const MERGE_SPACE: Complexity;
    fn scratch_size(n: usize) -> usize; // 0 = in-place
}
```

`scratchspace_lib::AuxiliaryMerge<A: Auxiliary>` consumes one and **is** a `RotationMerge`
(`scratch_size` delegates to `A`; `merge` runs a logged half-copy when the buffer holds the left run,
else the bufferless rotation merge) plus `HasTimeBounds`/`HasSpace`/`HasStability` (inherited from
`A`). So it drops straight into the `Category::Merge` emit (`run_rotation_merge::<Ty>`).

**The law (asserted by the crate tests):** merging two sorted halves yields the full multiset, sorted
and **stable** (equal keys keep original relative order), for every backing. All movement is via the
logger — nothing else touches the array.

**Backings as the composition axis:** `FullBuffer` (`O(n)`), `HalfBuffer` (`O(n/2)`), `NoBuffer`
(`O(1)`). `NoBuffer` is a first-class member (bufferless rotation merge), not an error path.

## Deliberately out of scope

- **Swap-based in-array buffer and keys.** Built, see As built. They are not `Scratchspace` methods, they are the `internal_size` of an `Auxiliary` and the `Keys` axis.
- **Stable-quicksort partition space.** Different animal (`O(log n)` counting/bits). Not part of this.

## Phasing

1. **`scratchspace_lib` component crate (DONE).** The trait + `InArray` + `ExternalFixed<512>` + a
   `buffered_merge`/`merge_runs` core that consumes any `Scratchspace` + self-check tests asserting
   the law on both backings (incl. heavy-duplicate/already-sorted inputs and the `capacity < want`
   degraded path). Standalone; no `ALGORITHMS` wiring.
2. **Round out backings + a real consumer (DONE).** Added `ExternalDyn` (runtime √n) and `NoScratch`
   (capacity-0, bufferless) — the same law/sort battery runs over all four. Added
   `bottom_up_merge_sort<S>`, so the trait composes into an actual stable sort. *The bufferless path was
   an insertion merge here and is the rotation merge now, see As built.*
3. **The `Keys` sibling axis (DONE).** The `Keys` role in `array_vis_bench_traits`, backed by `InArrayKeys` and `NoKeys` in `scratchspace_lib`.
4. **`block_merge_sort_lib` family (DONE).** `BlockMergeSort<Buffer, Keys>` with an inherent `sort` and the four annotation impls. WikiSort is `<InArrayBuffer, InArrayKeys>`, GrailSort with a buffer is `<FixedBuffer, InArrayKeys>`. A buffer or key shortage falls down the ladder and is logged with a `mark`.
5. **Registration and visualisation (DONE).** Six entries under `merge sorts / block`, and the buffer region event, `SortLog::Region` with `hold_region` and `release_region`, which the renderer shades.

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
- **The visualiser only needs the region event.** The trait and merges are correctness visible under any
  logger; the buffer region shading is the only consumer side surface, and the renderer has it.

## Pointers

- New home: `crates/components/scratchspace_lib/` (sits beside `reverse_storage_lib`,
  `gap_distribution_lib`), and `crates/families/block_merge_sort_lib/` for the sort.
- Merge consumers land in `crates/families/merge_sort_lib/` (already has `rotation_merge.rs`,
  `auxiliary_merge.rs`) or a new `crates/families/block_merge_sort_lib/`.
- ABI: `SortLogger` (`sort_logger`), `DirectSort` + `Has*` traits (`array_vis_bench_traits`).
- Research: the deep-research report (WikiSort README + Ch.3, Mrrl/GrailSort.h, Huang–Langston 1992,
  Kim–Kutzner 2008). Rotation building blocks already in-repo: `crates/rotations/*`.

## As built: the keys axis, the block merge sort, the region event

Shipped 2026-10-28, six entries and 1587 in the catalog. Where it deviates from the plan above, the
deviation is stated.

**The bufferless merge.** `scratchspace_lib::bufferless_merge` is symMerge, the repo's
`SmallerSideRotationMerge`, over `ReversalRotation`: `O(n)` comparisons, `O(n log n)` moves, no
storage. `AuxiliaryMerge<NoBuffer>`, the nightmare sort's `merge_runs` and every local merge that
lacks a buffer call it. The plan asked for block rolling here. Block rolling with no buffer is
`O(n sqrt n)` moves, worse than symMerge, so the real block rolling is in the block merge sort,
where a buffer makes it linear. The only complexity consts that changed are `NoBuffer::MERGE_WORST`
(`N_SQUARED` to `N_LOG_N`) and the nightmare sort's `WORST` and `AVERAGE` (`N_SQUARED` to
`N_LOG_SQUARED`, the bound of its zero aux member), which moved four rows of `catalog.golden`.
`HalfBuffer` declares `N1` for a merge whose left run fits and falls back to the rotation merge
for a skewed one, so its declared bound was never the worst case and is unchanged.

**Keys.** `array_vis_bench_traits::role::Keys`, with `InArrayKeys` and `NoKeys` in
`scratchspace_lib::keys`. `extract_distinct` gathers the first occurrence of each of the first
`want` distinct values at the front, sorted, the rest keeping their order, in `O(n + want^2)`
moves, and `sort_region` is an insertion sort; both are in the trait module because an in array
buffer is carved by the same call. The tags are WikiSort's, not GrailSort's. `tag_block` swaps a
key into the second element of a block, so the tag travels with the block, `find_block` takes the
lowest tag in the window, which is the next block of the left run in its original order, and the
same `tag_block` releases it. After every merge the key region is exactly as it was, and `restore`
insertion sorts it once at the end. GrailSort keeps a parallel key array and rotates it with the
window on every roll, `O(k)` a roll. `NoKeys` is the shape that never rolls a block: it wants no
keys and a merge under it is a buffered merge or the rotation merge.

**Buffers.** `Auxiliary::internal_size(n)` is new beside `scratch_size(n)`: cells carved out of
the array, a buffer of distinct elements the merge swaps through. `InArrayBuffer` carves
`isqrt(n)`, `FixedBuffer` is 512 external cells. Their spec role is `BlockBuffer`, not
`Auxiliary`, so the `auxiliary<...>` merge kinds do not grow, and `bb_none` is `NoBuffer` again
under that role. A fixed buffer cannot hold a block once `isqrt(n)` passes 512, so its
`MERGE_WORST` is `N_LOG_N` and the sort over it is `N_LOG_SQUARED`.

**The sort.** `BlockMergeSort<Buffer, Keys>`. `sort` carves the keys and the buffer as one run of
distinct elements, insertion sorts runs of 8, merges bottom up, then restores the keys, sorts the
scrambled buffer and slides the carved run back into the rest, `O(n + m^2)` moves. A merge whose
left run fits the stash is parked and merged whole. A longer one is `roll_merge` when there are
keys, and otherwise the rotation merge. `roll_merge` cuts the left run into blocks, rolls them
through the right run by block swaps and merges each block with the elements it lands among, as
the comment in `rolling.rs` derives. `Stash` is where the left run of a local merge waits: swapped
into the in array buffer, copied into the external one, or left in place with the rotation merge.
The bounds compose: the weaker of the keys and the buffer axes times `log n`, so WikiSort is
`N_LOG_N` and every shape with a missing axis is `N_LOG_SQUARED`.

**The ladder.** With fewer distinct elements than the keys and the buffer want, `settle` drops the
buffer first, then cuts the key count down to what was found, and with fewer than four keeps none.
Every fall is one `mark` that says what was found and what is kept, and every shape that asks for
nothing has nothing to drop. A fixed buffer that the blocks outgrow, `isqrt(n)` past 512 with keys,
logs one mark too, since the block merges then rotate their local merges.

**The region event.** `SortLog::Region { name, start, len, kind, held }` with `RegionKind` either
`Buffer` or `Keys`, emitted by `hold_region` and `release_region`. The sort holds the keys and the
carved buffer after the carve, the external buffer around the merges, and releases each after its
final sort. `sort_vis` shades a held buffer blue and a held key region amber, under a comparison
highlight.
