# Block merge sort family catalog fragment, owned by block_merge_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "block_merge.spec"`. One driver with five slots: the buffer, a `BlockBuffer`
# (in array or external, each sized by a `BufferSize`, or none, declared in
# scratchspace.spec), the keys, a `Keys` (in array or none), the `Rotation`
# every rotation of the sort runs, the `Extraction` that carves the keys and
# the buffer (declared in extraction.spec), and the `MergeStrategy` of the
# local merge where neither the buffer nor the keys reach (declared in
# merge.spec). A buffer of sqrt in the array with in array keys is WikiSort, an
# external one of 512 with in array keys is GrailSort with its static buffer,
# and of sqrt with its dynamic one, each carving by the scan and merging by the
# smaller side.

component block_sort
  type     BlockMergeSort<{buffer}, {keys}, {rot}, {extract}, {merge}>
  label    block sort<buffer: {buffer}, keys: {keys}, rotation: {rot}, extraction: {extract}, merge: {merge}>
  provides Sort
  adaptive true
  menu     merge sorts / block
  slot     buffer BlockBuffer
  slot     keys Keys
  slot     rot Rotation
  slot     extract Extraction
  slot     merge MergeStrategy
end
