# `src`

Source of `extraction_lib`. The crate README one level up says what it is for.

- `lib.rs`: the `Extraction` fillers, `ScanExtraction`, `PrefixSortExtraction` and `PartitionExtraction`, and the rotation merge sort they share.
- `partition.rs`: the `StablePartition` fillers `RotationPartition` and `SlidingPartition`, and the buffered pass they share.
- `blocks.rs`: what the block partitions share, the trim of the ends in place, the ordering of the blocks by ranks written into the elements, and the last rotation.
- `encoded.rs`: the `StablePartition` filler `EncodedPartition`, the linear one, which makes its blocks with a buffer.
- `masked.rs`: the `StablePartition` fillers `MaskedPartition` and `MaskedTablePartition`, which make their blocks by permuting windows of the region with bit masks kept in registered words and need no buffer.
- `test_util.rs`: the tagged item, the generator and the battery the module tests share.
