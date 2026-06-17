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
component part_left_left
  type     LeftLeftPartition
  label    left-left
  provides Partition
  project  pivot PivotSingle
  uses     partition_lomuto::LeftLeftPartition
end

component part_left_right
  type     LeftRightPartition
  label    left-right
  provides Partition
  project  pivot PivotSingle
  uses     partition_hoare::LeftRightPartition
end

component part_moving_pivot
  type     MovingPivot
  label    moving pivot
  provides Partition
  project  pivot PivotSingle
  uses     partition_moving_pivot::MovingPivot
end

component part_moving_pivot_v3
  type     MovingPivotV3<ReversalRotation>
  label    moving pivot v3
  provides Partition
  project  pivot PivotSingle
  uses     partition_moving_pivot_v3::MovingPivotV3 rotation_reversal::ReversalRotation
end

component part_three_way
  type     ThreeWay
  label    three-way
  provides Partition
  project  pivot PivotSingle
  uses     partition_three_way::ThreeWay
end

component part_block
  type     Block
  label    block
  provides Partition
  project  pivot PivotSingle
  uses     partition_block::Block
end

component part_dual
  type     DualPivotPartition
  label    dual
  provides Partition
  project  pivot PivotDual
  uses     quick_sort_lib::DualPivotPartition
end

# ════════════════════════ STANDARD SMALL SORTS ═══════════════════════════
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

# ════════════════════════ DEFERRED SMALL SORTS ═══════════════════════════
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

# ════════════════════════ SLOTTED DRIVERS ════════════════════════════════
component quick_sort
  type     QuickSort<{partition}, {pivot}, {small_sort}>
  label    quick sort<part: {partition}, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts
  uses     quick_sort_lib::QuickSort
  slot     partition Partition
  slot     pivot Pivot
  slot     small_sort SmallSort
end

component deferred_quick_sort
  type     DeferredQuickSort<{partition}, {pivot}, {small_sort}>
  label    deferred quick sort<part: {partition}, pivot: {pivot}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts
  uses     quick_sort_lib::DeferredQuickSort
  slot     partition Partition
  slot     pivot Pivot
  slot     small_sort DeferredSmallSort
end
