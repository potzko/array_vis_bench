# Implementation plan: move pivot under the partition (quicksort family re-model)

Status: **Steps 1 + 2 (Rust) DONE (2026-06-18); only the catalog naming cutover remains (deferred
to structural-nav).** Decided direction 2026-06-17. Separate concern from the enumeration strategies
([enumeration_strategies.md](enumeration_strategies.md)), but they compose nicely.

## Progress

- **Step 1 — separate pivoted / unpivoted (DONE 2026-06-18).** Added
  [`PivotlessQuickSort<P, SS>`](../../crates/families/quick_sort_lib/src/quick_sort.rs) and
  [`DeferredPivotlessQuickSort<P, DSS>`](../../crates/families/quick_sort_lib/src/deferred_quick_sort.rs):
  a quicksort driver that carries **no pivot vocabulary** — the partition self-splits, the sort
  supplies an empty pivot slice. Thin delegators over the shared driver (`QuickSort<P, NoPivot, SS>`
  internally) so there's one recursion to maintain, plus fresh compositional annotations (no
  `PivotQuality` axis — a self-splitting partition is O(log N) depth unconditionally). `HeapExtract`
  quicksort dropped its fake `NoPivot`; quick_heap correctness tests migrated + green.
  The name is exactly right: **`Pivotless` = the *sort* declares no pivot** (the partition still
  may, internally). That sets up step 2.
- **Step 2 — pivot under the partition (Rust DONE 2026-06-18).**
  [`Pivoted<Scheme, Sel>`](../../crates/families/quick_sort_lib/src/pivoted.rs) impls
  `PartitionScheme` with `N_PIVOTS = 0` (picks pivots via `Sel` internally) + the compositional
  annotations (`WORST = sum(Scheme, Sel)`, `DEGENERATES = Sel::DEGENERATES`, …). The recursion was
  flipped: the single pivot-free driver now lives in `PivotlessQuickSort` /
  `DeferredPivotlessQuickSort`; `QuickSort<P, V, SS>` and `DeferredQuickSort<P, V, DSS>` are now thin
  **shims** = `PivotlessQuickSort<Pivoted<P, V>, SS>` (the V-carrying *recursion* is retired; the
  type name stays so the catalog is untouched). The pivotless drivers read degeneracy off the
  partition via `PivotQuality` (`HeapExtract` gained `DEGENERATES = false`). All 448 quicksorts pass
  correctness; a new `spec_catalog` test `pivoted_form_matches_the_shim_complexity` locks the
  shim≡Pivoted equivalence so the future rename is pure. Full workspace green (catalog still 1493).
  **Remaining:** the catalog *naming* cutover (emit `QuickSort<Pivoted<…>, SS>` directly, drop the
  `QuickSort<P,V,SS>` shim + the `project pivot`/`Partition[pivot=p]` spec coupling) — deferred to
  the structural-nav work, as planned.

## The change

Today the pivot is a **peer type-param of the sort**:
```
QuickSort<P: PartitionScheme, V: PivotInput, SS: SmallSort>   // pivot V sits beside the partition P
DeferredQuickSort<P, V, DSS>
```
and partitions take pivot **indices at runtime** (`PartitionScheme::partition(arr, pivots: &[usize], …)`,
`const N_PIVOTS`). So `QuickSort` drives pivot selection (`V` computes indices) and hands them to `P`.

The pivot belongs to the *partition*, not the sort. Quicksort should just be an **indirection over a
self-pivoting partition**:
```
QuickSort<Partition, SmallSort>     // no pivot param
DeferredQuickSort<Partition, SmallSort>
```
- a **pivot-based** partition owns its selector — e.g. `Pivoted<LeftLeftPartition, FirstElement>`
  (a wrapper that picks pivots via the selector, then runs the scheme).
- a **pivotless** partition (`HeapExtractPartition`) has no pivot at all and no longer has to fake one.

## Step-2 feasibility assessment (2026-06-18) — clean, with one caveat + one real cost

The mechanism: a **`Pivoted<Scheme, Sel>` wrapper that itself impls `PartitionScheme` with
`N_PIVOTS = 0`** (NOT a new `Partitioner` trait — that was the earlier sketch). It picks pivots
via `Sel` internally, then runs `Scheme::partition` with them; externally it needs zero pivots:

```rust
impl<Scheme: PartitionScheme, Sel: PivotInput> PartitionScheme for Pivoted<Scheme, Sel> {
    const NAME: &'static str = Scheme::NAME;
    const N_PIVOTS: usize = 0;
    const SCRATCH_LEN: usize = Scheme::SCRATCH_LEN;
    fn partition<T, U, V>(arr, logger, _pivots, scratch, visitor) {
        let mut p = [0usize; 2];
        Sel::pick(arr, logger, &mut p);
        Scheme::partition(arr, logger, &p[..Sel::N], scratch, visitor);
    }
}
```

**Key consequence: this composes with step 1's `PivotlessQuickSort` driver directly.** Both a
pivot-based and a pivotless quicksort become `PivotlessQuickSort<SelfSplittingPartition, SS>`:
- pivotless: `PivotlessQuickSort<HeapExtract<…>, SS>`
- pivoted:   `PivotlessQuickSort<Pivoted<LeftLeftPartition, FirstElement>, SS>`

So step 1 is NOT throwaway — `PivotlessQuickSort` becomes THE driver and the V-carrying
`QuickSort<P, V, SS>` can be retired (eventual rename `PivotlessQuickSort` → `QuickSort`). The
driver never sees a pivot; every partition is self-contained.

### The four things that flow pivot→sort today, and whether each propagates cleanly

1. **Complexity / stability / degeneracy — CLEAN.** This is the "backward propagation" the design
   worried about, and it's tidy compositional const-fn forwarding. `Pivoted` combines its parts and
   forwards degeneracy up; the driver then reads everything off the partition:
   ```rust
   impl HasTimeBounds for Pivoted<Scheme, Sel> { const WORST = sum(Scheme::WORST, Sel::WORST); … }
   impl PivotQuality  for Pivoted<Scheme, Sel> { const DEGENERATES = Sel::DEGENERATES; }
   impl HasStability  for Pivoted<Scheme, Sel> { const STABLE = Scheme::STABLE && Sel::STABLE; }
   ```
   The driver's worst-case-depth formula moves from `V::DEGENERATES` to `Part::DEGENERATES`
   (`HeapExtract` hardcodes `false`; `Pivoted` forwards `Sel`). `QuickSort` becomes fully
   pivot-agnostic. **No mess.**
2. **Arity (`Sel::N == Scheme::N_PIVOTS`) — STAYS SPEC-ENFORCED (the caveat).** The earlier plan
   claimed arity becomes "structural" — that was over-promised. Making it a Rust type-level
   equality needs `where Sel::N == Scheme::N_PIVOTS`, i.e. `generic_const_exprs` (unstable). Not
   worth it. Reality: today the arity isn't type-enforced *either* (the driver just trusts slice
   lengths agree — it's the SPEC's `project pivot` + `Partition[pivot=p]` that prevents illegal
   pairings). With `Pivoted` the same coupling just moves one level down into the `pivoted`
   component's pivot sub-slot (sub-slot typed to the matching-arity pivot role). **Not a regression,
   not the advertised win — neutral.**
3. **Naming / menu labels — the REAL cost.** Pivot stops being a flat top-level slot and renders
   nested under the partition: emitted type `QuickSort<Pivoted<LeftLeftPartition, FirstElement>, …>`,
   label `quick sort<part: pivoted<left-left, first>, small: …>`. Emitted COUNTS stay (448) but the
   variant NAMES shift, breaking name-pinned golden assertions. The picker needs a recursive
   `partition → (pivoted) scheme → selector` drill-down. **This is the same problem the structural-nav
   plan already scopes** ([noble-mixing-patterson](../../.claude/plans/noble-mixing-patterson.md) /
   the deferred structural-nav work) — pivot-under-partition is its motivating use case. Land them
   together.

### Verdict
Rust-side propagation is **clean** (compositional forwarding; no new trait — `Pivoted` is just a
`PartitionScheme` with `N_PIVOTS = 0`). The mess is entirely on the **spec/registry/picker/naming**
side, and it's the structural-recursive-nav work that's already planned separately. Recommendation:
do the Rust `Pivoted` + driver-retirement first (self-contained, all behind existing tests), then
fold the naming/nav change in with the structural-nav plan rather than inventing a parallel one.

## Why (original motivation)

- Matches the semantics: pivot is a property of pivot-based partitions, period.
- `HeapExtractPartition` stops pretending to carry a pivot (the thing that motivated this).
- Makes the query clean and the rest-fill sugar honest: `quick_sort<partition = LL<pivot = _>, small = _>`
  nests pivot naturally; `quick_sort<_>` just fills partition+small.
- (NOTE: the earlier "arity coupling disappears / becomes structural" claim is corrected above —
  it stays spec-enforced, just localized into the `pivoted` component.)

## Affected surface

- **`quick_sort_lib`**: `QuickSort<P, V, SS>` → `QuickSort<Part, SS>`; same for `DeferredQuickSort`
  and the `HasTimeBounds/HasSpace/HasStability` impls (drop the `V` param, read complexity from
  `Part`).
- **New `Pivoted<Scheme, Selector>`** partition wrapper (likely in `quick_sort_lib` or a small crate):
  implements a "self-pivoting partition" interface QuickSort calls; internally uses `Selector` to pick
  pivots then runs `Scheme::partition`. Bound enforces `Selector::N == Scheme::N_PIVOTS` (this is where
  the old arity coupling now lives — in the type, not the query). Less invasive than making every
  scheme crate generic.
- **Partition crates** (`partition_lomuto/hoare/three_way/block`): unchanged (still `PartitionScheme`
  taking indices); they get wrapped by `Pivoted`. `HeapExtractPartition` already self-contained →
  implements the self-pivoting interface directly.
- **`quick.spec`**: `quick_sort` driver drops the `pivot` slot. Pivot-based partition components become
  `pivoted<scheme, pivot>` (a pivot sub-slot); heap-extract has no pivot slot. Delete the
  `project pivot …` lines and the query's `p`/`part` shared-var coupling.
- **Consistency follow-on** (decide separately): `quick_select_lib`'s `RecursiveQuickSelect<P, V>` /
  `IterativeQuickSelect<P, V>` and `fun_sorts` `quick_surrender` use the same `P, V` shape +
  the `QSPivotSingle/Dual/None` coupling I built in Phase 1. Re-modeling those the same way would
  delete that coupling too — but it's a bigger blast radius; do quick_sort first, then decide.

## Risk / open questions

- **Medium-high Rust refactor.** Every `QuickSort<…>` instantiation re-threads; the dual-pivot
  `CombinedSelector<a, b>` now nests under `Pivoted`. Monomorphisation/perf unchanged
  (`Pivoted<LL, FirstElement>` is still one concrete type).
- `Pivoted<Scheme, Selector>` wrapper vs making each scheme generic over its selector — wrapper is
  less invasive; confirm it composes with the dual-pivot partition (`DualPivotPartition`, `N_PIVOTS=2`).
- Apply to quick_select / quick_surrender now (consistency) or stage it?
- Emitted variant **names/labels** will shift (pivot now renders under the partition) — update the
  count tests + any name-pinned assertions.

## Pointers
- `crates/families/quick_sort_lib/src/quick_sort.rs` (`QuickSort<P, V, SS>`),
  `deferred_quick_sort.rs`; `array_vis_bench_traits/src/role/partition.rs` (`PartitionScheme`,
  `N_PIVOTS`); `crates/partitions/*`; `crates/families/quick_sort_lib/quick.spec`.
- Related: Phase-1 quick-select coupling in `project_first_class_kinds` (memory) +
  `crates/families/quick_select_lib/quick_select.spec`.
