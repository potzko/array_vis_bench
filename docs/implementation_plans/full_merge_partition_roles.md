# Full merge: one partition role-system (quick + quick-select + cyclent)

Status: **#1 DONE (2026-07-01).** All consumer counts preserved (quick 408,
quick-select 56, cyclent 16, quick-heap 32); Partition kind 51→67 (+16 last-based
partitions, as designed); catalog total 1544→1560. Roles 11→6. Full workspace green
(spec_catalog 66, spec_core+pest 97, all family suites). Ready to commit; #2 next.

## Outcome vs plan
- Deleted roles: QSPartition, QSPivot, QSPivotSingle, QSPivotDual, QSPivotNone,
  CyclentPart. Kept/added: PartScheme(5), PartPivot(6, +last), PartSchemeDual(1),
  PartPivotDual(37), PartitionKind(67), HeapExtractPart(4), + `SelectPartition` —
  quick-select's broader slot role (pivoted composites + heap-extract), NOT foreseen
  in the pitch but needed because `resolve()` role-checks EVERY slot filler.
- One Rust addition beyond the pitch: `DualPivoted<Sel>` type ALIAS (distinct head
  for the dual composite — `pk_single`/`pk_dual` share the `Pivoted` head and
  `resolve()`'s slot-role check needs `pk_dual` referenceable). No new impls.
- Registry made parse-order deterministic (`iter()` was HashMap order — a latent
  build-nondeterminism bug); first-wins now resolves `combined`/`pk_single`/
  `ninther_dual` correctly. `pkp_*` declared first in pivot specs so PartPivot wins
  the pivot type-heads.
- Names changed (as flagged): cyclent `left-left pointer`→`left-left`; quick-select
  nests pivot-under-partition (`recursive<left-left<first>>`); quick-surrender cascades.

## Ground truth (verified)
- Drivers all consume the visitor contract; `RecursiveQuickSelect<P,V>` /
  `IterativeQuickSelect<P,V>` use no pivot positions → can take ONE self-pivoting
  `PartitionScheme` param (mirror `CanonicalQuickSort`). `CyclentSort<P>` already
  takes one `PartitionScheme` (hardcodes pivot=last). `CyclentPart` is a spec-only
  label.
- Set-diff & dedup are by resolved `type_expr` STRING ([solve.rs:392,406]),
  independent of which component built the node. `pk_single`/`pk_dual` share the
  `Pivoted<{scheme},{pivot}>` head AND the `{scheme}<{pivot}>` label template →
  a subtrahend like `Pivoted<_, LastElement>` resolves to the right strings even
  though the head resolves (first-wins) to `pk_single`. **No new Rust head needed.**
- Explicit App fillers are NOT role-checked (only holes are, via `enumerate`),
  so a query may place any `PartitionScheme` in a `part` slot; rustc backstops.
- heap-extract (4) are flat `HeapExtract<AryPair<A>, DH>` composites sharing the
  `HeapExtract` head → need a ROLE to enumerate → `HeapExtractPart`.

## Roles: 11 → 6
DELETE: `QSPartition`, `QSPivot`, `QSPivotSingle`, `QSPivotDual`, `QSPivotNone`,
`CyclentPart`. KEEP/add: `PartScheme` (5), `PartPivot` (6, **+last**),
`PartSchemeDual` (1), `PartPivotDual` (37), `PartitionKind` (67),
**`HeapExtractPart`** (4, heap-extract's home). Each atom declared ONCE per crate.

## Atom/count math
- PartPivot = first, median3, middle, ninther, median-of-medians, **last** = 6
- PartPivotDual = combined<PartPivot,PartPivot> (36) + ninther_dual (1) = 37
- PartitionKind = pk_single (5×6=30) + pk_dual (1×37=37) = **67** (was 51)

## Consumer queries (all existing features)
- **quicksort** — preserve 408. `part = _ - { Pivoted<_, LastElement>,
  Pivoted<DualPivotPartition, CombinedSelector<LastElement,_>>,
  Pivoted<DualPivotPartition, CombinedSelector<_,LastElement>> }` → 67-16 = 51.
  × 4 small × 2 = 408.
- **Partition kind** (kinds.avbs) — GROW to **67** (include the 16 new last-based
  standalone partitions; heap-extract stays out). Report the +16.
- **quick-select** — preserve 56. Split each strategy into pivoted + heap-extract:
  - pivoted (24) = `where( Pivoted<scheme=(_-{MovingPivot}), pivot=(_-{MedianOfMedians})>
    | Pivoted<DualPivotPartition, CombinedSelector<FirstElement,FirstElement>>
    | Pivoted<DualPivotPartition, CombinedSelector<MiddleElement,MiddleElement>>
    | Pivoted<DualPivotPartition, CombinedSelector<FirstElement,LastElement>>
    | Pivoted<DualPivotPartition, NintherDualPivot> )` (20 single + 4 dual)
  - he (4) = `part = he` with `he: HeapExtractPart = _`
  - 24 + 4 = 28 per strategy × 2 = 56.
- **cyclent** — preserve 16. `CyclentSort<part = _ - {MovingPivot}>` over PartScheme (4).
- **quick-surrender** (fun_sorts) — nests quick-select; split piv/he too. 168 preserved.

## NAME CHANGES (unavoidable consequence of shared labels)
- cyclent: `cyclent sort<left-left pointer>` → `cyclent sort<left-left>` (pks label,
  no "pointer").
- quick-select: `quick select: recursive<left-left pointer, first>` →
  `quick select: recursive<left-left<first>>` (pivot NESTS under partition, like quick).
  he: `<heap extract: binary iterative, no pivot>` → `<heap extract: binary iterative>`.
- quick-surrender names cascade the same way.

## Rust changes
- quick_select_lib: `RecursiveQuickSelect<P: PartitionScheme>` /
  `IterativeQuickSelect<P>` (drop the `V` param; `V::pick` gone, pass `&[]` pivots).
  Composable impls read from `P` alone (like `Pivoted`). ONLY Rust change.

## Golden updates (spec_catalog/src/lib.rs)
PartitionKind/Partition-kind 51→67; quick 408 (preserved); quick-select 56
(preserved, names nested); cyclent 16 (names deloused); quick-surrender 168
(names cascade); quick-heap 32 (unchanged, heap-extract role renamed); COUNT_TOTAL
recompute; facet/name tests for quick-select + cyclent.

## Stages (build-green each)
1. Rust driver (quick_select_lib) — single-param.
2. Specs: +pkp_last; delete QS*/CyclentPart components; heap-extract→HeapExtractPart;
   quick_select.spec drivers single `part` slot.
3. Queries: quick.avbs curation; quick_select.avbs split; fun_sorts.avbs cyclent +
   quick-surrender; kinds.avbs grow.
4. Goldens.
5. Build + iterate; adversarial verify; commit.

## #2 finite() — DONE (2026-07-01, commit e6edd80)
`finite( _ )` = occurs-check enumeration (a tree is kept iff no component appears in
its own subtree). `QValue::Finite`; `enumerate::enumerate_finite` threads the
ancestor path and skips self-nesting; solve/cardinality give it an EXACT finite card
so a cyclic role passes the budget guard. Composes as a slot filler. Test: a
binary-recursive role that's refused under a bare hole is bounded to 2 by finite().

## #2 dogfood — DONE (2026-07-02, commit 7140ba0; "minimal proof" scope)
Wired the first genuinely-recursive role cycle into the catalog and bounded it with
`finite()`: `SetDeepHeapify → (set arm) RecSelect → RecSelPart(open heap-extract) →
RecDeepHeapify → …`. finite() unfolds the cycle EXACTLY ONCE → the 2 principled
variants (`set quick heap sort<set<select<heap extract: binary<{iterative|recursive}>>>>`).
- Rust: `SetQuickHeapSort<DH>` alias (`QuickHeapSort<Binary, DH, NoSmallSort>`) +
  `SetSelect<P>` alias (`RecursiveQuickSelect<P>`) — distinct type-heads so the family
  is referenceable without colliding with the classic-qhs / quick-select heads.
- Roles: RecDeepHeapify (Iterative/Recursive leaves + `rdh_set`), SetDeepHeapify
  (`rdh_set` only — so ONLY the recursive arm emits; the leaves are classic qhs),
  RecSelect (`rsel`), RecSelPart (`hx_open`, the open `HeapExtract<AryPair<Binary>,{dh}>`).
- **Curation decision (was the blocker): "minimal proof"** — the inner select's part
  is a DEDICATED narrow role `RecSelPart` (just the open heap-extract), NOT the 71-member
  SelectPartition, so finite emits 2, not hundreds. arity baked Binary, `max_input 128`.
- `depth 12` on quick_heap.avbs makes finite() LOAD-BEARING (a bare hole grows with
  depth; finite pins to 2 — tested). quick-heap 32→34, catalog total 1562.

## Follow-ups (not done)
- The other 3 enumeration strategies (`affine`/`sample`/`spread`) remain unimplemented
  (only `finite` was built). See [[project_enumeration_strategies]].
- The set-quick-select family could expand (more arities / the RecursiveSet arm /
  wider inner-select partitions) — deferred; "minimal proof" was the chosen scope.
