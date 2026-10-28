# `block_merge_sort_lib`

Block merge sort, `BlockMergeSort<Buffer, Keys>`. A stable merge sort that does
its merges with a small buffer instead of an `O(n)` scratch array: the left run
is cut into blocks of about `sqrt(n)`, the blocks are rolled through the right
run by block swaps, and each block is merged with the elements it lands
among, using the buffer as swap space. The blocks are tagged by a key region
of distinct elements, so a rolled block still says which block it was.

Two axes. The buffer is an `Auxiliary` backing: `InArrayBuffer` carves it out
of the array, `FixedBuffer` is a 512 cell external array, `NoBuffer` has none
and the local merges rotate. The keys are a `Keys` backing: `InArrayKeys` or
`NoKeys`, which never rolls a block. The in array buffer with in array keys is
WikiSort, the fixed buffer with in array keys is GrailSort with a buffer.

When the array has too few distinct elements for the buffer or the keys the
sort falls down the ladder (buffer dropped, key count cut, keys dropped,
rotation merges only) and logs each fall with a `mark`.

Its catalog component is declared in `block_merge.spec`, the buffer and keys fillers in
`scratchspace_lib`'s fragment.

Query `block_merge.avbs` builds: `BlockMergeSorts`, six entries.
