# Level heap sort family catalog fragment, owned by level_heap_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "level_heap.spec"`. One driver with three slots: the buffer, a `BlockBuffer`
# in the array or outside it by a `BufferSize` (declared in scratchspace.spec),
# the `StablePartition` the level build selects with (declared in
# extraction.spec) and the `Rotation` it rotates with.

component level_heap_sort
  type      LevelHeapSort<{buffer}, {part}, {rot}>
  label     level heap sort<buffer: {buffer}, partition: {part}, rotation: {rot}>
  provides  Sort
  menu      heap sorts / level
  slot      buffer BlockBuffer
  slot      part StablePartition
  slot      rot Rotation
end
