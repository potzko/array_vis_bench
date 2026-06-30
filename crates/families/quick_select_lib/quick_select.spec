# Quick-SELECT family catalog fragment — OWNED by quick_select_lib.
#
# The first NON-SORT first-class kind in the catalog: `category QuickSelect`
# drivers emit through `spec_emit`'s quick-select body (role-trait `select` ABI,
# `run_quick_select_with_input`, `CorrectnessSuite`/`SelectSuite` battery) and the
# `Selection<X>` AVBS wrapper marks `X` emittable at role `QuickSelect`.
#
# CURATED axes (NOT quick-sort's full set): the 56 PROVEN combinations from the
# standalone `quick_select_registry`, no more. Quick-select recurses into only the
# region holding `target`, so it reuses quicksort's partition/pivot TYPES but a
# DIFFERENT, smaller axis set — `last` is in (quicksort drops it), `median of
# medians` / `MovingPivot` / the 25-pair `combined` fan-out are out (MovingPivot
# in particular risks the partition non-convergence class). Distinct `QS*` roles
# keep this axis set isolated from the quick-sort query in the shared registry.
#
# ARITY COUPLING (structural, same mechanism as quick.spec): single partitions
# `project pivot QSPivotSingle`, the dual `project QSPivotDual`, the pivotless
# heap-extract partitions `project QSPivotNone`; the query threads a shared pivot
# var `p` + `QSPartition[pivot = p]`, so `P::N_PIVOTS == V::N` always holds and an
# arity-illegal `QuickSelect<P, V>` is never built.
#
# COUNT per strategy: 4 single-part × 5 single-piv + 1 dual-part × 4 dual-piv
# + 4 heap-extract-part × 1 no-piv = 20 + 4 + 4 = 28. × 2 strategies = 56.

# ════════════════════════════ SINGLE PIVOTS ════════════════════════════

# ════════════════════════════ DUAL PIVOTS (curated 4) ═══════════════════

# ════════════════════════════ NO PIVOT (heap-extract) ═══════════════════

# ════════════════════════════ SINGLE PARTITIONS ═════════════════════════

# ════════════════════════════ DUAL PARTITION ════════════════════════════

# ═════════════════════ HEAP-EXTRACT PARTITIONS (pivotless) ═══════════════

# ════════════════════════════ SLOTTED DRIVERS ═══════════════════════════
component qs_recursive
  type     RecursiveQuickSelect<{partition}, {pivot}>
  label    quick select: recursive<{partition}, {pivot}>
  provides QuickSelect
  category QuickSelect
  menu     recursive
  uses     quick_select_lib::RecursiveQuickSelect
  slot     partition QSPartition
  slot     pivot QSPivot
end

component qs_iterative
  type     IterativeQuickSelect<{partition}, {pivot}>
  label    quick select: iterative<{partition}, {pivot}>
  provides QuickSelect
  category QuickSelect
  menu     iterative
  uses     quick_select_lib::IterativeQuickSelect
  slot     partition QSPartition
  slot     pivot QSPivot
end
