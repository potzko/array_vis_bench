# `extraction_lib`

How a block merge sort carves its keys and buffer, the `Extraction` axis, and
the stable partitions, the `StablePartition` axis, that one of its fillers
selects with.

An extraction gathers `want` distinct elements of a region at its front,
sorted, each the first occurrence of its value, the rest keeping their order
among equals. `ScanExtraction` is GrailSort's and WikiSort's one scan, the
keys gathered in a block that rides behind it, `O(n + c^2)` moves for `c`
keys. `PrefixSortExtraction` sorts the first `c` and a tenth elements by
rotation merges first, so the scan sweeps them without a move and only carries
on into the rest when the prefix ran short of distinct values.
`PartitionExtraction<P>` takes the keys from the smallest values: the sorted
prefix is its sample, the sample's element at the target's rank is the pivot,
and `P` partitions the rest by it, moving the pivot along the sample and
partitioning again when the selection falls outside its window, up to three
times; the selection is then sorted and the scan compacts it. Its point is the
end of the sort, where keys below the rest slide back in without a move, and
its price is the partition of the whole array.

A stable partition brings the elements of a region below a pivot value to its
front, each side keeping its order, or the elements not above it, and the two
together split a region three ways, below, equal and above, which a stable
selection needs so that a region of one value has somewhere to stop. `RotationPartition` halves the region,
partitions each half and rotates the middle, with a one pass buffered
partition for a region the rotation's scratch holds: `O(n)` comparisons,
`O(n log n)` moves, `O(n log (n / s))` with `s` cells of scratch.
`SlidingPartition` gathers the matches in one block that rides behind the
scan, each run of elements that stay costing one rotation of the block past
it: `O(n)` comparisons and `O(n + m r)` moves for `m` matches in `r` runs,
linear when they are few or clustered and quadratic at worst.

Its catalog components are declared in `extraction.spec`.
