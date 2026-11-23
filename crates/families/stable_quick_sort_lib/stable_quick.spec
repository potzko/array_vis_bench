# Stable quick sort family catalog fragment, owned by stable_quick_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "stable_quick.spec"`. One driver with two slots: the `StablePartition` that
# splits a region around the pivot (declared in extraction.spec) and the
# `Rotation` it rotates with.

component stable_quick_sort
  type      StableQuickSort<{part}, {rot}>
  label     stable quick sort<partition: {part}, rotation: {rot}>
  provides  Sort
  menu      stable quick sorts
  slot      part StablePartition
  slot      rot Rotation
end
