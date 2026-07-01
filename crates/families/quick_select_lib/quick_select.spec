# Quick-SELECT family catalog fragment — OWNED by quick_select_lib.
#
# The first NON-SORT first-class kind in the catalog: `category QuickSelect`
# drivers emit through `spec_emit`'s quick-select body (role-trait `select` ABI,
# `run_quick_select_with_input`, `CorrectnessSuite`/`SelectSuite` battery) and the
# `Selection<X>` AVBS wrapper marks `X` emittable at role `QuickSelect`.
#
# ONE partition role-system (shared with quick): each driver slots a single
# self-pivoting `part` — a `Pivoted<scheme, pivot>` composite (`PartitionKind`) or a
# pivotless `HeapExtract` (`HeapExtractPart`). The old quick-only `QS*` roles are
# gone; quick-select CURATES the shared population in its query (quick_select.avbs)
# instead of via private role-membership.
#
# CURATED axis set (NOT quick-sort's full set): quick-select keeps `last` (quicksort
# drops it) and drops `median of medians` pivot / `MovingPivot` scheme / the 25-pair
# `combined` dual fan-out (MovingPivot in particular risks partition non-convergence).
# The curation now lives in the query as set-diff + a small dual union.
#
# COUNT per strategy: 4 single-scheme × 5 single-pivot (PartScheme−{moving pivot} ×
# PartPivot−{median of medians}) + 4 curated dual + 4 heap-extract = 20 + 4 + 4 = 28.
# × 2 strategies = 56.

# ════════════════════════════ SINGLE PIVOTS ════════════════════════════

# ════════════════════════════ DUAL PIVOTS (curated 4) ═══════════════════

# ════════════════════════════ NO PIVOT (heap-extract) ═══════════════════

# ════════════════════════════ SINGLE PARTITIONS ═════════════════════════

# ════════════════════════════ DUAL PARTITION ════════════════════════════

# ═════════════════════ HEAP-EXTRACT PARTITIONS (pivotless) ═══════════════

# ════════════════════════════ SLOTTED DRIVERS ═══════════════════════════
component qs_recursive
  type     RecursiveQuickSelect<{part}>
  label    quick select: recursive<{part}>
  provides QuickSelect
  category QuickSelect
  menu     recursive
  uses     quick_select_lib::RecursiveQuickSelect
  slot     part SelectPartition
end

component qs_iterative
  type     IterativeQuickSelect<{part}>
  label    quick select: iterative<{part}>
  provides QuickSelect
  category QuickSelect
  menu     iterative
  uses     quick_select_lib::IterativeQuickSelect
  slot     part SelectPartition
end
