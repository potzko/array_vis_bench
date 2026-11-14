# `level_heap_sort_lib`

Level heap sort, `LevelHeapSort<B, M, P, R>`: a stable heap sort through a
buffer of elements smaller than the array, over a buffer `B`, carved from
the array or allocated beside it and sized by a `BufferSize`, the marks `M`
of its dead cells, a stable partition `P` and a rotation `R`.

A heap sort is unstable because a pop moves the element at the array's end
to the root, across every equal of it. Out of place there is nothing to
move: the pop takes the root and the hole sinks instead, pulling the smaller
live child up, ties to the left, until it reaches a leaf, which dies. In the
pre-order layout both rising moves keep equal elements in order, the left
child being the adjacent cell and the right child rising over the left
subtree only when strictly smaller than its root, so with a build by
selection, which never passes an equal either, the pops come out stable with
no stamp and no index. The price is a buffer to hold the heap and a mark per
cell for the dead leaves.

The level ordered heap is what makes the buffer smaller than the array.
Stable selection at the power of two boundaries, from the largest down,
leaves every level a rank interval with equals in original order, and a
level's sorted output is exactly its own cell range. So each level is sorted
out of place in turn, swapped into the buffer, heaped there and popped back
into its cells, each dead leaf taking the buffer element the output cell
held, and the buffer only ever has to hold the largest level. A level larger
than the buffer is a range in its own right, levelled again and sorted level
by level, the ranges halving with every depth, so a buffer of `B` cells costs
`O(n log (n / B))` comparisons on top of the sort's `O(n log n)`, and `B`
can be the square root the block sorts carve. A carved buffer is distinct,
so an unstable heap sort puts it back in order at the end, and it slides
home like a block sort's.

The marks are a flag per buffer cell, and `M` is where they are kept: a byte
or a bit of an aux array, or two more distinct elements of the array per
cell, carved along with the buffer, in order while the cell is live and
swapped once it is dead. Carved marks are read by a comparison and written
by a swap, and with a carved buffer the sort allocates nothing.

The time is the level build's. Its comparisons are `O(n log n)` expected,
but a stable partition with no scratch moves `n log n`, and one runs over
every element at each of the `log (n / B)` depths, so the sort is
`O(n log^2 n)` expected over an in place rotation and a buffer smaller than
linear. It is `O(n log n)` with a linear buffer, reached in a constant
number of depths, or over a rotation whose scratch is linear, which the
partition then makes one pass through. The selection is a quickselect, so
the worst case is `n` partitions of the range, `O(n^2 log n)` in place.

Its catalog component is declared in `level_heap.spec` and built by
`level_heap.avbs`.
