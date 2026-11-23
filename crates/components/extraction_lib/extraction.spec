# Extraction fragment, owned by extraction_lib and gathered by spec_catalog
# through `[package.metadata.array_vis_bench] spec = "extraction.spec"`. The
# `Extraction` slot of the block merge sort, how it carves its keys and buffer:
# the one scan of GrailSort and WikiSort, a sorted prefix under the scan, or the
# smallest values selected by a `StablePartition` around a pivot read off that
# prefix. The partitions are by rotation, by a block riding behind the scan, or
# by blocks put in order with their ranks written into the elements, which are
# gathered through a buffer or made by bit masks over windows of the region.

component ex_scan
  type     ScanExtraction
  label    scan
  provides Extraction
end

component ex_prefix_sort
  type     PrefixSortExtraction
  label    prefix sort
  provides Extraction
end

component ex_partition
  type     PartitionExtraction<{part}>
  label    partition<{part}>
  provides Extraction
  slot     part StablePartition
end

component sp_rotation
  type     RotationPartition
  label    rotation
  provides StablePartition
end

component sp_sliding
  type     SlidingPartition
  label    sliding
  provides StablePartition
end

component sp_encoded
  type     EncodedPartition
  label    encoded
  provides StablePartition
end

component sp_masked
  type     Masked<{walk}>
  label    masked<{walk}>
  provides StablePartition
  slot     walk MaskWalk
end

# The walk of the masked partition, how the cycles of a window's permutation
# find each element's place: by popcounts of the window's mask as the cycle is
# walked, in 290 bytes of locals, or from a table of 16 bit places filled before
# any element moves, in 2210 bytes. The table is the faster walk, and the popcount
# is the one that spends bits where the table spends cells.

component mw_popcount
  type     Popcount
  label    popcount
  provides MaskWalk
end

component mw_table
  type     Tabulated
  label    table
  provides MaskWalk
end
