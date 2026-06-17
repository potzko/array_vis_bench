# Merge-sort family catalog fragment — OWNED by merge_sort_lib.
#
# The spec compiler (`spec_catalog`) gathers this fragment via
# `[package.metadata.array_vis_bench] spec = "merge.spec"` and merges it into the
# unified registry. Component names are prefixed `merge_` / `mrm_` / `mrot_` for
# global uniqueness. SmallSort role components (ss_size1/ss_size2/ss_insertion32/
# ss_network16) are REUSED from quick.spec — do NOT redeclare them here. `uses`
# paths resolve in the consumer (`spec_catalog`), which links the leaf crates.

# ── Classic merge sorts (type head + SmallSort slot + two bool consts) ──────────
component merge_top_down
  type     TopDownMergeSort<{small_sort}, {ping_pong}, {early_exit}>
  label    merge sort<small: {small_sort}, pp={ping_pong}, ee={early_exit}>
  provides Sort
  category Sort
  menu     merge sorts / classic / top-down
  uses     merge_sort_lib::top_down::TopDownMergeSort
  slot     small_sort SmallSort ss_size1
  const    ping_pong false values false true
  const    early_exit false values false true
end

component merge_bottom_up
  type     BottomUpMergeSort<{small_sort}, {ping_pong}, {early_exit}>
  label    bottom-up merge sort<small: {small_sort}, pp={ping_pong}, ee={early_exit}>
  provides Sort
  category Sort
  menu     merge sorts / classic / bottom-up
  uses     merge_sort_lib::bottom_up::BottomUpMergeSort
  slot     small_sort SmallSort ss_size1
  const    ping_pong false values false true
  const    early_exit false values false true
end

component merge_naive
  type     NaiveMergeSort<{small_sort}>
  label    naive merge sort<small: {small_sort}>
  provides Sort
  category Sort
  menu     merge sorts / classic / naive
  uses     merge_sort_lib::naive::NaiveMergeSort
  slot     small_sort SmallSort ss_size1
end

component merge_natural
  type     NaturalMergeSort<{ping_pong}, {early_exit}>
  label    natural merge sort<pp={ping_pong}, ee={early_exit}>
  provides Sort
  category Sort
  adaptive true
  menu     merge sorts / classic / natural
  uses     merge_sort_lib::natural::NaturalMergeSort
  const    ping_pong false values false true
  const    early_exit false values false true
end

# ── Rotation-based merge sorts (nested RotationMerge slot over Rotation) ────────
# EARLY_EXIT is pinned false (the legacy families bake `false`), so it is a const
# taking its default — not exposed as an axis.
component merge_rot_top_down
  type     TopDownRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>
  label    rotation merge sort<small: {small_sort}, merge: {rmerge}>
  provides Sort
  category Sort
  menu     merge sorts / rotation / top-down
  uses     merge_sort_lib::rotation::TopDownRotationMergeSort
  slot     small_sort SmallSort ss_size1
  slot     rmerge RotationMerge mrm_naive
  const    early_exit false
end

component merge_rot_bottom_up
  type     BottomUpRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>
  label    bottom-up rotation merge sort<small: {small_sort}, merge: {rmerge}>
  provides Sort
  category Sort
  menu     merge sorts / rotation / bottom-up
  uses     merge_sort_lib::rotation::BottomUpRotationMergeSort
  slot     small_sort SmallSort ss_size1
  slot     rmerge RotationMerge mrm_naive
  const    early_exit false
end

# ── RotationMerge strategies (each wraps a Rotation) ────────────────────────────
component mrm_naive
  type     NaiveRotationMerge<{rot}>
  label    naive<{rot}>
  provides RotationMerge
  uses     merge_sort_lib::rotation_merge::NaiveRotationMerge
  slot     rot Rotation mrot_reversal
end

component mrm_smaller_side
  type     SmallerSideRotationMerge<{rot}>
  label    smaller-side<{rot}>
  provides RotationMerge
  uses     merge_sort_lib::rotation_merge::SmallerSideRotationMerge
  slot     rot Rotation mrot_reversal
end

# ── Rotation components (11; uses resolve to the rotation leaf crates) ───────────
component mrot_reversal
  type     ReversalRotation
  label    reversal
  provides Rotation
  uses     rotation_reversal::ReversalRotation
end

component mrot_juggling
  type     JugglingRotation
  label    juggling
  provides Rotation
  uses     rotation_juggling::JugglingRotation
end

component mrot_gries_mills
  type     GriesMillsRotation
  label    gries-mills
  provides Rotation
  uses     rotation_gries_mills::GriesMillsRotation
end

component mrot_auxiliary
  type     AuxiliaryRotation
  label    auxiliary
  provides Rotation
  uses     rotation_auxiliary::AuxiliaryRotation
end

component mrot_grail
  type     GrailRotation
  label    grail
  provides Rotation
  uses     rotation_grail::GrailRotation
end

component mrot_bridge
  type     BridgeRotation
  label    bridge
  provides Rotation
  uses     rotation_bridge::BridgeRotation
end

component mrot_contrev
  type     ContrevRotation
  label    contrev
  provides Rotation
  uses     rotation_contrev::ContrevRotation
end

component mrot_drill
  type     DrillRotation
  label    drill
  provides Rotation
  uses     rotation_drill::DrillRotation
end

component mrot_helix
  type     HelixRotation
  label    helix
  provides Rotation
  uses     rotation_helix::HelixRotation
end

component mrot_piston
  type     PistonRotation
  label    piston
  provides Rotation
  uses     rotation_piston::PistonRotation
end

component mrot_trinity
  type     TrinityRotation
  label    trinity
  provides Rotation
  uses     rotation_trinity::TrinityRotation
end
