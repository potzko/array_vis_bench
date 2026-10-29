# Block merge sort family catalog fragment, owned by block_merge_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "block_merge.spec"`. One driver with three slots: the buffer, a `BlockBuffer`
# (in array or external, each sized by a `BufferSize`, or none, declared in
# scratchspace.spec), the keys, a `Keys` (in array or none), and the `Rotation`
# every rotation of the sort runs. A buffer of sqrt in the array with in array
# keys is WikiSort, an external one of 512 with in array keys is GrailSort with
# its static buffer, and of sqrt with its dynamic one.

component block_sort
  type     BlockMergeSort<{buffer}, {keys}, {rot}>
  label    block sort<buffer: {buffer}, keys: {keys}, rotation: {rot}>
  provides Sort
  adaptive true
  menu     merge sorts / block
  slot     buffer BlockBuffer
  slot     keys Keys
  slot     rot Rotation
end
