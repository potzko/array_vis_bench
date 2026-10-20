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
  menu     merge sorts / classic / top-down
  slot     small_sort SmallSort
  const    ping_pong values false true
  const    early_exit values false true
end

component merge_bottom_up
  type     BottomUpMergeSort<{small_sort}, {ping_pong}, {early_exit}>
  label    bottom-up merge sort<small: {small_sort}, pp={ping_pong}, ee={early_exit}>
  provides Sort
  menu     merge sorts / classic / bottom-up
  slot     small_sort SmallSort
  const    ping_pong values false true
  const    early_exit values false true
end

component merge_naive
  type     NaiveMergeSort<{small_sort}>
  label    naive merge sort<small: {small_sort}>
  provides Sort
  menu     merge sorts / classic / naive
  slot     small_sort SmallSort
end

component merge_natural
  type     NaturalMergeSort<{ping_pong}, {early_exit}>
  label    natural merge sort<pp={ping_pong}, ee={early_exit}>
  provides Sort
  adaptive true
  menu     merge sorts / classic / natural
  const    ping_pong values false true
  const    early_exit values false true
end

# ── Rotation-based merge sorts (nested RotationMerge slot over Rotation) ────────
# EARLY_EXIT is pinned false (the legacy families bake `false`): a literal only
# const that the rotation queries pass as `false`, not exposed as an axis.
component merge_rot_top_down
  type     TopDownRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>
  label    rotation merge sort<small: {small_sort}, merge: {rmerge}>
  provides Sort
  menu     merge sorts / rotation / top-down
  slot     small_sort SmallSort
  slot     rmerge RotationMerge
  const    early_exit
end

component merge_rot_bottom_up
  type     BottomUpRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>
  label    bottom-up rotation merge sort<small: {small_sort}, merge: {rmerge}>
  provides Sort
  menu     merge sorts / rotation / bottom-up
  slot     small_sort SmallSort
  slot     rmerge RotationMerge
  const    early_exit
end

# ── RotationMerge strategies (each wraps a Rotation); also the first class merge kinds ──
component mrm_naive
  type     NaiveRotationMerge<{rot}>
  label    naive<{rot}>
  provides RotationMerge MergeKind
  slot     rot Rotation
end

component mrm_smaller_side
  type     SmallerSideRotationMerge<{rot}>
  label    smaller-side<{rot}>
  provides RotationMerge MergeKind
  slot     rot Rotation
end

