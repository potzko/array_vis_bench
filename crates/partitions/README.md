# partitions

One crate per partition scheme. Each implements `PartitionScheme`, reports
its unsorted ranges to a visitor, and provides the `PartScheme` role. The
composites that pair a scheme with a pivot live in `quick_sort_lib`.

- `partition_block`: block partition with offset buffers.
- `partition_hoare`: left right crossing scan.
- `partition_lomuto`: left left single pointer scan.
- `partition_moving_pivot`: the pivot walks inward.
- `partition_three_way`: below, equal, above.
