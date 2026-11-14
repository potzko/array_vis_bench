# `src`

Source of `extraction_lib`. The crate README one level up says what it is for.

- `lib.rs`: the `Extraction` fillers, `ScanExtraction`, `PrefixSortExtraction` and `PartitionExtraction`, and the rotation merge sort they share.
- `partition.rs`: the `StablePartition` fillers, `RotationPartition`, `SlidingPartition` and `BlockPartition`.
- `test_util.rs`: the tagged item and generator the module tests share.
