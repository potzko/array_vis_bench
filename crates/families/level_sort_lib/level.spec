# Level sort family catalog fragment, owned by level_sort_lib and gathered by
# `spec_catalog` through `[package.metadata.array_vis_bench] spec = "level.spec"`.
# One driver with two slots: the `StablePartition` its selections split with
# (declared in extraction.spec) and the `Rotation` that partition rotates with.

component level_sort
  type      LevelSort<{part}, {rot}>
  label     level sort<partition: {part}, rotation: {rot}>
  provides  Sort
  menu      heap sorts / level
  slot      part StablePartition
  slot      rot Rotation
end
