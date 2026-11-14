# Extraction fragment, owned by extraction_lib and gathered by spec_catalog
# through `[package.metadata.array_vis_bench] spec = "extraction.spec"`. The
# `Extraction` slot of the block merge sort, how it carves its keys and buffer:
# the one scan of GrailSort and WikiSort, a sorted prefix under the scan, or the
# smallest values selected by a `StablePartition` around a pivot read off that
# prefix. The partitions are by rotation, by a block riding behind the scan, or
# by whole blocks through a workspace the caller lends, which the extraction
# does not have, so the block partition is not one of its fillers' partitions.

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

component sp_block
  type     BlockPartition
  label    block
  provides StablePartition
end
