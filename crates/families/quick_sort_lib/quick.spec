# Quick-sort family catalog fragment — OWNED by this crate.
#
# Gathered by `spec_catalog` via `[package.metadata.array_vis_bench] spec =
# "quick.spec"`. This is the SLOTTED decomposition: one `quick_sort<partition,
# pivot, small_sort>` driver (+ a deferred sibling) whose slots the query fills.
#
# The pivot / partition / small-sort COMPONENTS this family composes are declared
# here as the quick family's contract, even though several of their TYPES live in
# sibling leaf crates (`partition_lomuto::LeftLeftPartition`, …). The `uses` paths
# are plain text resolved in the consumer (`spec_catalog`), so this fragment adds
# NO compile-time dependency from `quick_sort_lib` onto those leaves.
#
# ARITY COUPLING (structural): single-pivot partitions `project pivot
# PivotSingle`, the dual partition `project pivot PivotDual`; the query threads a
# shared pivot var `p` + `Partition[pivot = p]` refinement, so an arity-illegal
# pairing is never built. Emits the SAME 448 concrete types as the baked port.

# ════════════════════════════ PIVOT SELECTORS ════════════════════════════

# The dual "combined" pair: two single selectors composed → 5 × 5 = 25 pairs.
component combined
  type     CombinedSelector<{a}, {b}>
  label    combined<{a}, {b}>
  provides Pivot PivotDual
  uses     quick_sort_lib::CombinedSelector
  slot     a PivotSingle p_first
  slot     b PivotSingle p_middle
end

# The baked "ninther thirds" dual selector (1 variant).
component ninther_dual
  type     NintherDualPivot
  label    ninther
  provides Pivot PivotDual
  uses     quick_sort_lib::NintherDualPivot
end

# ════════════════════════════ PARTITIONS ════════════════════════════

component part_dual
  type     DualPivotPartition
  label    dual
  provides Partition
  project  pivot PivotDual
  uses     quick_sort_lib::DualPivotPartition
end

# ════════════════════════ STANDARD SMALL SORTS ═══════════════════════════

# ════════════════════════ DEFERRED SMALL SORTS ═══════════════════════════

# ════════════════════════ SLOTTED DRIVERS ════════════════════════════════
component quick_sort
  type     CanonicalQuickSort<Pivoted<{partition}, {pivot}>, {small_sort}>
  label    quick sort<part: {partition}, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts
  uses     quick_sort_lib::CanonicalQuickSort quick_sort_lib::Pivoted
  slot     partition Partition
  slot     pivot Pivot
  slot     small_sort SmallSort
end

component deferred_quick_sort
  type     DeferredCanonicalQuickSort<Pivoted<{partition}, {pivot}>, {small_sort}>
  label    deferred quick sort<part: {partition}, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts
  uses     quick_sort_lib::DeferredCanonicalQuickSort quick_sort_lib::Pivoted
  slot     partition Partition
  slot     pivot Pivot
  slot     small_sort DeferredSmallSort
end
component beap_dps_first
  type     CombinedSelector<FirstElement, FirstElement>
  label    first
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement
end

component beap_dps_middle
  type     CombinedSelector<MiddleElement, MiddleElement>
  label    middle
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end

component beap_dps_ninther
  type     NintherDualPivot
  label    ninther 1/3 + 2/3
  provides BeapDualPivot
  uses     quick_sort_lib::NintherDualPivot
end

component qs_dp_first_first
  type     CombinedSelector<FirstElement, FirstElement>
  label    first / first
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement
end

component qs_dp_mid_mid
  type     CombinedSelector<MiddleElement, MiddleElement>
  label    middle / middle
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end

component qs_dp_first_last
  type     CombinedSelector<FirstElement, LastElement>
  label    first / last
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement pivot_last::LastElement
end

component qs_dp_ninther
  type     NintherDualPivot
  label    ninther 1/3 + 2/3
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::NintherDualPivot
end

component qs_part_dual
  type     DualPivotPartition
  label    dual pivot
  provides QSPartition
  project  pivot QSPivotDual
  uses     quick_sort_lib::DualPivotPartition
end

component pk_pivoted
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end
