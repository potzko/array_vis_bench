# `extraction_lib`

How a block merge sort carves its keys and buffer, the `Extraction` axis, and
the stable partitions, the `StablePartition` axis, that one of its fillers
selects with.

A stable partition brings the elements of a region below a pivot value to its
front, each side keeping its order. `RotationPartition` halves the region,
partitions each half and rotates the middle, with a one pass buffered
partition for a region the rotation's scratch holds: `O(n)` comparisons,
`O(n log n)` moves, `O(n log (n / s))` with `s` cells of scratch.
`SlidingPartition` gathers the matches in one block that rides behind the
scan, each run of elements that stay costing one rotation of the block past
it: `O(n)` comparisons and `O(n + m r)` moves for `m` matches in `r` runs,
linear when they are few or clustered and quadratic at worst.

Its catalog components are declared in `extraction.spec`.
