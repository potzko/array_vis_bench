# `extraction_lib`

How a block merge sort carves its keys and buffer, the `Extraction` axis, and
the stable partitions, the `StablePartition` axis, that one of its fillers
selects with and that a stable quick sort is built on.

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
`EncodedPartition` is linear whatever the shape, `O(n)` moves and
comparisons with `O(log n)` cells of scratch, the idea of Katajainen and
Pasanen, "Stable minimum space partitioning in linear time" (BIT, 1992), and
of aphitorite's Logsort partition. One scan gathers the region into blocks of
one type each, with a buffer of a block for the back elements; a block swap
then puts the blocks in order, and the one thing a block swap cannot do,
remember which block was which, is written into the elements: the `j`th block
of the minority type and the `j`th of the other type swap the cells at the set
bits of `j`, which the pivot can read back since the two elements it swaps are
never alike, so no element has to be distinct. The scratch is a block, eight
times the logarithm of the region, which the partition lets go of before its
last rotation, so it is allocated with the rotation's once, by the role's
`scratch_size`; a region that scratch holds is partitioned in one buffered
pass instead. The front elements a region starts with and the back ones it
ends with are in place, and are compared once and left, so a pass with nothing
to move costs its comparisons alone.
`MaskedPartition` makes the same blocks with no buffer at all. The region goes
through in windows of up to a thousand elements, each window's types held as a
bit mask in sixteen words, and a window is permuted in place into
whole blocks of the front type, whole blocks of the back type, and then the
partial blocks that start the next window. An element's place is worked out
from popcounts of the mask below it, the cycles of the permutation are walked
with one element carried in a local, and every element that moves is written
once. The mask, its counts and the positions done are the fixed size arrays a C
version keeps on the stack, and here they are registered words, three arrays
laid side by side of 16, 17 and 16 words, whatever the element type is: the
`StablePartition` role declares them as `SCRATCH_SECTIONS`, the sort allocates
them once through the logger and every write to them is logged. It asks for no
scratch of elements. `MaskedTablePartition` is the same with the places of a
window filled into a table first, a word a position, which the cycle walk then
reads instead of counting bits, at 16, 17 and 1024 words. Both share the trim, the
ordering of the blocks and the last rotation with the encoded partition, in
`blocks.rs`. The walk is the `walk` slot of `Masked<K>` in the catalog, filled
by `Popcount` or `Tabulated`, so both are catalogued, as `masked<popcount>`
and `masked<table>`.

`PartitionExtraction<P>` asks for the words of its partition and hands them on,
and the other fillers ask for none.

`BlockPartition` works through a workspace its caller lends, a buffer of
elements and a mark per cell: the scan streams the matches into one half of
the buffer and the rest into the other, a half that fills goes back into
the cells the scan has emptied as a whole block, and the blocks are then
moved into order along the cycles of their permutation, read off a mark per
block for its kind and kept track of by a second: `O(n)` comparisons and
moves for a region up to a quarter of the buffer's square, a longer one
halved and rotated. Without a workspace it is the rotation partition, so it
is not offered to the partition extraction.

Its catalog components are declared in `extraction.spec`.
