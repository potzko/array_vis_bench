# `src`

Source of `extraction_lib`. The crate README one level up says what it is for.

- `lib.rs`: the `Extraction` fillers, `ScanExtraction` and `PrefixSortExtraction`, and the rotation merge sort they share.
- `partition.rs`: the `StablePartition` fillers, `RotationPartition` and `SlidingPartition`.
- `test_util.rs`: the tagged item and generator the module tests share.
