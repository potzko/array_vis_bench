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
component qs_p_first
  type     FirstElement
  label    first
  provides QSPivot QSPivotSingle
  uses     pivot_first::FirstElement
end

component qs_p_middle
  type     MiddleElement
  label    middle
  provides QSPivot QSPivotSingle
  uses     pivot_middle::MiddleElement
end

component qs_p_last
  type     LastElement
  label    last
  provides QSPivot QSPivotSingle
  uses     pivot_last::LastElement
end

component qs_p_median3
  type     MedianOfThree
  label    median of 3
  provides QSPivot QSPivotSingle
  uses     pivot_median3::MedianOfThree
end

component qs_p_ninther
  type     Ninther
  label    ninther
  provides QSPivot QSPivotSingle
  uses     pivot_ninther::Ninther
end

# ════════════════════════════ DUAL PIVOTS (curated 4) ═══════════════════
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

# ════════════════════════════ NO PIVOT (heap-extract) ═══════════════════
component qs_nopivot
  type     NoPivot
  label    no pivot
  provides QSPivot QSPivotNone
  uses     array_vis_bench_traits::NoPivot
end

# ════════════════════════════ SINGLE PARTITIONS ═════════════════════════
component qs_part_left_left
  type     LeftLeftPartition
  label    left-left pointer
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_lomuto::LeftLeftPartition
end

component qs_part_left_right
  type     LeftRightPartition
  label    left-right pointer
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_hoare::LeftRightPartition
end

component qs_part_three_way
  type     ThreeWay
  label    three-way
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_three_way::ThreeWay
end

component qs_part_block
  type     Block
  label    block
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_block::Block
end

# ════════════════════════════ DUAL PARTITION ════════════════════════════
component qs_part_dual
  type     DualPivotPartition
  label    dual pivot
  provides QSPartition
  project  pivot QSPivotDual
  uses     quick_sort_lib::DualPivotPartition
end

# ═════════════════════ HEAP-EXTRACT PARTITIONS (pivotless) ═══════════════
component qs_part_he_bin_it
  type     HeapExtract<AryPair<Binary>, Iterative>
  label    heap extract: binary iterative
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Binary heap_sort_lib::Iterative
end

component qs_part_he_bin_re
  type     HeapExtract<AryPair<Binary>, Recursive>
  label    heap extract: binary recursive
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Binary heap_sort_lib::Recursive
end

component qs_part_he_ter_it
  type     HeapExtract<AryPair<Ternary>, Iterative>
  label    heap extract: ternary iterative
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Ternary heap_sort_lib::Iterative
end

component qs_part_he_ter_re
  type     HeapExtract<AryPair<Ternary>, Recursive>
  label    heap extract: ternary recursive
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Ternary heap_sort_lib::Recursive
end

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
