# Merge-sort family catalog fragment — OWNED by merge_sort_lib.
#
# The spec compiler (`spec_catalog`) gathers this fragment via
# `[package.metadata.array_vis_bench] spec = "merge.spec"` and merges it into the
# unified registry. Component names are prefixed `merge_` / `mrm_` for
# global uniqueness. SmallSort role components (ss_size1/ss_size2/ss_insertion32/
# ss_network16) are declared in the small sort leaf crates, do not redeclare them
# here. `uses` paths resolve in the consumer (`spec_catalog`), which links the leaf crates.

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

# ── RotationMerge strategies (each wraps a Rotation); also the first class merge kinds ──
component mrm_naive
  type     NaiveRotationMerge<{rot}>
  label    naive<{rot}>
  provides RotationMerge MergeKind
  category Merge
  uses     merge_sort_lib::rotation_merge::NaiveRotationMerge
  slot     rot Rotation rot_reversal
end

component mrm_smaller_side
  type     SmallerSideRotationMerge<{rot}>
  label    smaller-side<{rot}>
  provides RotationMerge MergeKind
  category Merge
  uses     merge_sort_lib::rotation_merge::SmallerSideRotationMerge
  slot     rot Rotation rot_reversal
end

