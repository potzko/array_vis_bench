# `src`

Source of `extraction_lib`. The crate README one level up says what it is for.

- `lib.rs`: the `Extraction` fillers, `ScanExtraction`, `PrefixSortExtraction` and `PartitionExtraction`, and the rotation merge sort they share.
- `partition.rs`: the `StablePartition` fillers `RotationPartition` and `SlidingPartition`, and the buffered pass they share.
- `blocks.rs`: what a block partition does once its blocks are made, the trim, the ordering of the blocks by ranks written into the elements, and the last rotation.
- `encoded.rs`: the `StablePartition` filler `EncodedPartition`, the linear one, which makes its blocks with a buffer.
- `masked.rs`: the `StablePartition` filler `MaskedPartition`, which makes its blocks by permuting windows of the region with bit masks and needs no buffer.
- `test_util.rs`: the tagged item, the generator and the battery the module tests share.
