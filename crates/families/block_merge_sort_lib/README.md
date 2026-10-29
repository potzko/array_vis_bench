# `block_merge_sort_lib`

Block merge sort, `BlockMergeSort<Buffer, Keys, Rotation>`. A stable merge sort that does
its merges with a small buffer instead of an `O(n)` scratch array: the left run
is cut into blocks of about `sqrt(n)`, the blocks are rolled through the right
run by block swaps, and each block is merged with the elements it lands
among, using the buffer as swap space. The blocks are tagged by a key region
of distinct elements, so a rolled block still says which block it was.

Three axes. The buffer is an `Auxiliary` backing: `InArrayBuffer` carves it out
of the array, `ExternalBuffer` is an aux array, `NoBuffer` has none and the
local merges rotate. The first two take a `BufferSize`: `Sqrt`, `TwoSqrt`,
`Log`, `Fixed<512>`, `Half` or `Sixteenth`. The keys are a `Keys` backing: `InArrayKeys` or `NoKeys`, which
never rolls a block. The rotation is any `Rotation`: it carves the keys and the
buffer, carries a short block in a block merge, does the rotation merges and
slides the carved elements back in. A rotation that needs scratch gets one aux
array for the whole sort. A buffer of `Sqrt` in the array with in array keys is
WikiSort, an external one of 512 with in array keys is GrailSort with its static
buffer, and of `Sqrt` with its dynamic one.

The size trades the merges against the carving. A buffer that keeps up with
`sqrt(n)` holds a block of every merge, so each merge is linear and the sort is
`O(n log n)`; a smaller one, `Log` or `Fixed<512>`, leaves the block merges
rotating, `O(n log^2 n)`.
Carving `c` cells and sorting them back in costs `O(n + c^2)` moves, so past
`sqrt(n)` an in array buffer no longer lowers the order of the merges and only
costs more to carve; half or a sixteenth of the array carved is `O(n^2)`. Outside the array the size is the space the sort
takes.

When the array has too few distinct elements for the buffer or the keys the
sort falls down the ladder (buffer dropped, key count cut, keys dropped,
rotation merges only) and logs each fall with a `mark`.

Its catalog component is declared in `block_merge.spec`, the buffer, size and keys fillers in
`scratchspace_lib`'s fragment.

Query `block_merge.avbs` builds: `BlockMergeSorts`, 286 entries: thirteen
buffers (in the array or external by six sizes, or none) by two key shapes by
eleven rotations.
