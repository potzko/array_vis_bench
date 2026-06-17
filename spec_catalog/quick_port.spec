# Quick-sort PORT catalog — reproduces the combo_codegen "quick sorts" families
# (every partition EXCEPT heap-extract) in the spec compiler, matching the old
# picker shape:
#
#   sorts → spec → quick sorts          → <partition> → pivot × small-sort
#   sorts → spec → deferred quick sorts  → <partition> → pivot × deferred-small
#
# Partition is a MENU sub-branch (baked into each entry component's type), not a
# facet; pivot + small-sort are the two faceted slots. Standard and deferred are
# two SEPARATE quick-sort families (the design call), differing only in driver
# (`QuickSort` vs `DeferredQuickSort`) and small-sort role (`SmallSort` vs
# `DeferredSmallSort`). Both provide role `Sort`.
#
# ARITY COUPLING (the spec headline, still enforced — just per-component now):
# a single-pivot partition's `pivot` slot is role `PivotSingle`, so it can ONLY
# be filled by the 5 single pivots; a dual partition's pivot is `PivotPair`
# (combined) or the baked `NintherDualPivot`. An arity-illegal pairing is
# structurally unrepresentable — never enumerated, never emitted.
#
# `uses` lines take multiple space-separated crate paths (e.g. the driver AND the
# baked partition). These are the canonical quick-sort registrations now — the
# combo registry is disconnected, so the labels carry no `spec::` marker.

# ════════════════════════════ PIVOT SELECTORS ════════════════════════════
# Single-pivot selectors (PivotInput::N = 1) — the 5 the old single families use.
component p_first
  type     FirstElement
  label    first
  provides Pivot PivotSingle
  uses     pivot_first::FirstElement
end

component p_middle
  type     MiddleElement
  label    middle
  provides Pivot PivotSingle
  uses     pivot_middle::MiddleElement
end

component p_median3
  type     MedianOfThree
  label    median of 3
  provides Pivot PivotSingle
  uses     pivot_median3::MedianOfThree
end

component p_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides Pivot PivotSingle
  uses     pivot_median_of_medians::MedianOfMedians
end

component p_ninther
  type     Ninther
  label    ninther
  provides Pivot PivotSingle
  uses     pivot_ninther::Ninther
end

# The dual-pivot "combined" pair: two single selectors composed. Its distinct
# `PivotPair` role keeps the dual "combined" branch separate from the baked
# "ninther 1/3 + 2/3" branch (which the old shape also splits as a sub-category).
# Enumerating its `pivot` slot recursively enumerates a × b = 25 pairs.
component combined
  type     CombinedSelector<{a}, {b}>
  label    combined<{a}, {b}>
  provides PivotPair
  uses     quick_sort_lib::CombinedSelector
  slot     a PivotSingle p_first
  slot     b PivotSingle p_middle
end

# ════════════════════════ STANDARD SMALL SORTS (SS) ═══════════════════════
component ss_size1
  type     Size1SmallSort
  label    size: 1
  provides SmallSort
  uses     small_sort_basic::Size1SmallSort
end

component ss_size2
  type     Size2SmallSort
  label    size: 2
  provides SmallSort
  uses     small_sort_basic::Size2SmallSort
end

component ss_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides SmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component ss_network16
  type     Network16SmallSort
  label    network: 16
  provides SmallSort
  uses     small_sort_network_16::Network16SmallSort
end

# ════════════════════ DEFERRED SMALL SORTS (DSS) ══════════════════════════
component dss_lin16
  type     DeferredInsertion<LinearInsertion, 16>
  label    deferred insertion: 16
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::LinearInsertion
end

component dss_lin32
  type     DeferredInsertion<LinearInsertion, 32>
  label    deferred insertion: 32
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::LinearInsertion
end

component dss_bin16
  type     DeferredInsertion<BinaryInsertion, 16>
  label    deferred binary insertion: 16
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::BinaryInsertion
end

component dss_bin32
  type     DeferredInsertion<BinaryInsertion, 32>
  label    deferred binary insertion: 32
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::BinaryInsertion
end

# ═══════════════ STANDARD FAMILY: QuickSort<P, V, SS> ══════════════════════
# Six single-pivot partition entry-points (pivot slot = PivotSingle).
component qs_std_left_left
  type     QuickSort<LeftLeftPartition, {pivot}, {small_sort}>
  label    quick sort<part: left-left, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / left-left pointer
  uses     quick_sort_lib::QuickSort partition_lomuto::LeftLeftPartition
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

component qs_std_left_right
  type     QuickSort<LeftRightPartition, {pivot}, {small_sort}>
  label    quick sort<part: left-right, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / left-right pointer
  uses     quick_sort_lib::QuickSort partition_hoare::LeftRightPartition
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

component qs_std_moving_pivot
  type     QuickSort<MovingPivot, {pivot}, {small_sort}>
  label    quick sort<part: moving pivot, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / moving pivot
  uses     quick_sort_lib::QuickSort partition_moving_pivot::MovingPivot
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

component qs_std_moving_pivot_v3
  type     QuickSort<MovingPivotV3<ReversalRotation>, {pivot}, {small_sort}>
  label    quick sort<part: moving pivot v3, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / moving pivot v3
  uses     quick_sort_lib::QuickSort partition_moving_pivot_v3::MovingPivotV3 rotation_reversal::ReversalRotation
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

component qs_std_three_way
  type     QuickSort<ThreeWay, {pivot}, {small_sort}>
  label    quick sort<part: three-way, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / three-way
  uses     quick_sort_lib::QuickSort partition_three_way::ThreeWay
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

component qs_std_block
  type     QuickSort<Block, {pivot}, {small_sort}>
  label    quick sort<part: block, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / block
  uses     quick_sort_lib::QuickSort partition_block::Block
  slot     pivot PivotSingle
  slot     small_sort SmallSort
end

# Dual-pivot entry-points (the old "dual pivot" branch splits combined vs ninther).
component qs_std_dual_combined
  type     QuickSort<DualPivotPartition, {pivot}, {small_sort}>
  label    quick sort<part: dual, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / dual pivot / combined
  uses     quick_sort_lib::QuickSort quick_sort_lib::DualPivotPartition
  slot     pivot PivotPair
  slot     small_sort SmallSort
end

component qs_std_dual_ninther
  type     QuickSort<DualPivotPartition, NintherDualPivot, {small_sort}>
  label    quick sort<part: dual, pivot: ninther, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts / dual pivot / ninther thirds
  uses     quick_sort_lib::QuickSort quick_sort_lib::DualPivotPartition quick_sort_lib::NintherDualPivot
  slot     small_sort SmallSort
end

# ═══════════ DEFERRED FAMILY: DeferredQuickSort<P, V, DSS> ══════════════════
# Mirror of the standard family: deferred driver, small-sort role DeferredSmallSort,
# nested under a separate "deferred quick sorts" menu branch.
component qs_def_left_left
  type     DeferredQuickSort<LeftLeftPartition, {pivot}, {small_sort}>
  label    deferred quick sort<part: left-left, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / left-left pointer
  uses     quick_sort_lib::DeferredQuickSort partition_lomuto::LeftLeftPartition
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_left_right
  type     DeferredQuickSort<LeftRightPartition, {pivot}, {small_sort}>
  label    deferred quick sort<part: left-right, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / left-right pointer
  uses     quick_sort_lib::DeferredQuickSort partition_hoare::LeftRightPartition
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_moving_pivot
  type     DeferredQuickSort<MovingPivot, {pivot}, {small_sort}>
  label    deferred quick sort<part: moving pivot, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / moving pivot
  uses     quick_sort_lib::DeferredQuickSort partition_moving_pivot::MovingPivot
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_moving_pivot_v3
  type     DeferredQuickSort<MovingPivotV3<ReversalRotation>, {pivot}, {small_sort}>
  label    deferred quick sort<part: moving pivot v3, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / moving pivot v3
  uses     quick_sort_lib::DeferredQuickSort partition_moving_pivot_v3::MovingPivotV3 rotation_reversal::ReversalRotation
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_three_way
  type     DeferredQuickSort<ThreeWay, {pivot}, {small_sort}>
  label    deferred quick sort<part: three-way, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / three-way
  uses     quick_sort_lib::DeferredQuickSort partition_three_way::ThreeWay
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_block
  type     DeferredQuickSort<Block, {pivot}, {small_sort}>
  label    deferred quick sort<part: block, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / block
  uses     quick_sort_lib::DeferredQuickSort partition_block::Block
  slot     pivot PivotSingle
  slot     small_sort DeferredSmallSort
end

component qs_def_dual_combined
  type     DeferredQuickSort<DualPivotPartition, {pivot}, {small_sort}>
  label    deferred quick sort<part: dual, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / dual pivot / combined
  uses     quick_sort_lib::DeferredQuickSort quick_sort_lib::DualPivotPartition
  slot     pivot PivotPair
  slot     small_sort DeferredSmallSort
end

component qs_def_dual_ninther
  type     DeferredQuickSort<DualPivotPartition, NintherDualPivot, {small_sort}>
  label    deferred quick sort<part: dual, pivot: ninther, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts / dual pivot / ninther thirds
  uses     quick_sort_lib::DeferredQuickSort quick_sort_lib::DualPivotPartition quick_sort_lib::NintherDualPivot
  slot     small_sort DeferredSmallSort
end
