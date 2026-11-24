# Level heap sort family catalog fragment, owned by level_heap_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "level_heap.spec"`. One driver with four slots: the buffer, a `BlockBuffer`
# in the array or outside it by a `BufferSize`, the `Marks` of its dead cells
# (both declared in scratchspace.spec), the `StablePartition` the level build
# selects with (declared in extraction.spec) and the `Rotation` it rotates with.

component level_heap_sort
  type      LevelHeapSort<{buffer}, {marks}, {part}, {rot}>
  label     level heap sort<buffer: {buffer}, marks: {marks}, partition: {part}, rotation: {rot}>
  provides  Sort
  menu      heap sorts / level
  slot      buffer BlockBuffer
  slot      marks Marks
  slot      part StablePartition
  slot      rot Rotation
end
