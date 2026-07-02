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

## #2 dogfood — SURFACED, NOT WIRED (needs a curation decision)
The genuine recursive cycle for the set-quick-select heap family:
`RecDeepHeapify → QuickSelect → SelectPartition → HeapExtract(open dh slot) →
RecDeepHeapify`. The Rust types compile (see quick_heap_sort_lib/tests/correctness.rs)
and `finite(_)` on `dh` bounds the recursion (unfolds once, terminates in a leaf DH).
Wiring needs: a `SetQuickHeapSort` alias (distinct head vs classic qhs), an OPEN
`HeapExtract<AryPair<{arity}>, {dh}>` providing SelectPartition, `SequentialSet`/
`RecursiveSet` deep-heapify components, a `RecDeepHeapify` role (Iterative/Recursive
leaves + the two set arms), a driver family + a `finite(_)` query.

**Blocker (product call):** the inner quick-select's `part` is `SelectPartition`
(67 pivoted + 4 heap-extract + the open one), so `finite(dh)` fans over ALL of them
→ hundreds of SLOW O(n²) recursive variants in the correctness battery. Needs
curation to a sane size — a narrow inner-select partition set + baked arity/small +
`max_input` cap. HOW MANY recursive variants to emit is the user's call.
