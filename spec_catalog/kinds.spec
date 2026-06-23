# First-class BUILDING-STONE KINDS catalog fragment — OWNED by spec_catalog.
#
# These are cross-leaf KINDS (rotations, small sorts, partitions, merges) with no
# single home crate, so they live next to the catalog that gathers them. Each is
# a standalone first-class algorithm with its own `category` (so spec_emit picks
# the per-kind body) and its OWN role (`RotationKind`/`SmallSortKind`/… — distinct
# from the same-named SLOT-FILLER roles the merge/quick families already use, so a
# `_` query over a kind never collides with another family's fillers).
#
# All the leaf crates are already in spec_catalog's dependency closure (they back
# the merge/quicksort/mergesort families); here they each ALSO surface as their
# own visualisable/testable entries under their kind's picker root.
#
# ════════════════════════════════ ROTATIONS ════════════════════════════════
# `RotationKind` role, `category Rotation`, `RotationAlg<_>` query (no slots).

component rot_auxiliary
  type     AuxiliaryRotation
  label    auxiliary
  provides RotationKind
  category Rotation
  uses     rotation_auxiliary::AuxiliaryRotation
end

component rot_bridge
  type     BridgeRotation
  label    bridge
  provides RotationKind
  category Rotation
  uses     rotation_bridge::BridgeRotation
end

component rot_contrev
  type     ContrevRotation
  label    contrev
  provides RotationKind
  category Rotation
  uses     rotation_contrev::ContrevRotation
end

component rot_drill
  type     DrillRotation
  label    drill
  provides RotationKind
  category Rotation
  uses     rotation_drill::DrillRotation
end

component rot_grail
  type     GrailRotation
  label    grail
  provides RotationKind
  category Rotation
  uses     rotation_grail::GrailRotation
end

component rot_gries_mills
  type     GriesMillsRotation
  label    gries-mills
  provides RotationKind
  category Rotation
  uses     rotation_gries_mills::GriesMillsRotation
end

component rot_helix
  type     HelixRotation
  label    helix
  provides RotationKind
  category Rotation
  uses     rotation_helix::HelixRotation
end

component rot_juggling
  type     JugglingRotation
  label    juggling
  provides RotationKind
  category Rotation
  uses     rotation_juggling::JugglingRotation
end

component rot_piston
  type     PistonRotation
  label    piston
  provides RotationKind
  category Rotation
  uses     rotation_piston::PistonRotation
end

component rot_reversal
  type     ReversalRotation
  label    reversal
  provides RotationKind
  category Rotation
  uses     rotation_reversal::ReversalRotation
end

component rot_trinity
  type     TrinityRotation
  label    trinity
  provides RotationKind
  category Rotation
  uses     rotation_trinity::TrinityRotation
end

# ════════════════════════════════ SMALL SORTS ═══════════════════════════════
# `SmallSortKind` role, `category SmallSort`, `SmallSortAlg<_>` query. A small
# sort is only correct up to its THRESHOLD, so each declares `max_input` = its
# threshold (caps the interactive picker; the battery is bounded by THRESHOLD via
# SmallSortSuite). Names are `sk_*` — quick.spec already owns `ss_*` (its slot
# fillers) in the global component namespace.
component sk_size1
  type     Size1SmallSort
  label    size: 1
  provides SmallSortKind
  category SmallSort
  max_input 1
  uses     small_sort_basic::Size1SmallSort
end

component sk_size2
  type     Size2SmallSort
  label    size: 2
  provides SmallSortKind
  category SmallSort
  max_input 2
  uses     small_sort_basic::Size2SmallSort
end

component sk_network16
  type     Network16SmallSort
  label    network: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_network_16::Network16SmallSort
end

component sk_ins_lin16
  type     InsertionSmallSort<LinearInsertion, 16>
  label    insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component sk_ins_lin32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component sk_ins_bin16
  type     InsertionSmallSort<BinaryInsertion, 16>
  label    binary insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end

component sk_ins_bin32
  type     InsertionSmallSort<BinaryInsertion, 32>
  label    binary insertion: 32
  provides SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end

# ════════════════════════════════ PARTITIONS ════════════════════════════════
# `PartitionKind` role, `category Partition`. A partition can't run standalone
# without a pivot, so each entry is a self-pivoting `Pivoted<Scheme, Sel>` (the
# wrapper from quick_sort_lib that fuses a PartitionScheme + PivotInput into a
# zero-external-pivot PartitionScheme). The `pk_pivoted` driver cross-products
# `PartScheme × PartPivot` → 5 single-pivot schemes × 6 single pivots = 30.
# Distinct roles `PartScheme`/`PartPivot` (quick.spec owns `Partition`/`Pivot`).

# ── partition schemes (single-pivot) ──
component pks_lomuto
  type     LeftLeftPartition
  label    left-left
  provides PartScheme
  uses     partition_lomuto::LeftLeftPartition
end

component pks_hoare
  type     LeftRightPartition
  label    left-right
  provides PartScheme
  uses     partition_hoare::LeftRightPartition
end

component pks_block
  type     Block
  label    block
  provides PartScheme
  uses     partition_block::Block
end

component pks_three_way
  type     ThreeWay
  label    three-way
  provides PartScheme
  uses     partition_three_way::ThreeWay
end

component pks_moving
  type     MovingPivot
  label    moving pivot
  provides PartScheme
  uses     partition_moving_pivot::MovingPivot
end

# ── partition pivots (single) ──
component pkp_first
  type     FirstElement
  label    first
  provides PartPivot
  uses     pivot_first::FirstElement
end

component pkp_middle
  type     MiddleElement
  label    middle
  provides PartPivot
  uses     pivot_middle::MiddleElement
end

component pkp_median3
  type     MedianOfThree
  label    median of 3
  provides PartPivot
  uses     pivot_median3::MedianOfThree
end

component pkp_median_med
  type     MedianOfMedians
  label    median of medians
  provides PartPivot
  uses     pivot_median_of_medians::MedianOfMedians
end

component pkp_ninther
  type     Ninther
  label    ninther
  provides PartPivot
  uses     pivot_ninther::Ninther
end

component pkp_last
  type     LastElement
  label    last
  provides PartPivot
  uses     pivot_last::LastElement
end

# ── driver: a self-pivoting partition (scheme × pivot) ──
component pk_pivoted
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end

# ════════════════════════════════ MERGES ════════════════════════════════════
# `MergeKind` role, `category Merge`. Merges have no role trait in the traits
# crate — `RotationMerge` lives in merge_sort_lib — so the emit inlines
# `merge_battery` (like sorts). Each merge family takes a rotation; the slot
# REUSES the `RotationKind` leaves above (as fillers their category is ignored).
# 2 families × 11 rotations = 22.
component mk_naive
  type     NaiveRotationMerge<{rot}>
  label    naive<{rot}>
  provides MergeKind
  category Merge
  uses     merge_sort_lib::rotation_merge::NaiveRotationMerge
  slot     rot RotationKind
end

component mk_smaller
  type     SmallerSideRotationMerge<{rot}>
  label    smaller-side<{rot}>
  provides MergeKind
  category Merge
  uses     merge_sort_lib::rotation_merge::SmallerSideRotationMerge
  slot     rot RotationKind
end
