# `stable_quick_sort_lib`

Stable quick sort, `StableQuickSort<P, R>`: a quick sort whose split is a
stable partition `P`, the `StablePartition` axis, rotating with a rotation
`R`.

A quick sort is unstable because its partition throws an element across the
equal ones it passes. A stable partition keeps each side in order, so the
sort needs no tag and no index, only the scratch the partition and the
rotation share, allocated once, and the fixed size words the partition keeps,
allocated once as well. The pivot is a copied value, the median of
three values and past 128 elements the median of three such medians, so
nothing is moved to choose it. The region splits into the elements below the
pivot and the rest, and the sort recurses into the smaller of the two and
loops on the larger, so the stack is `log n` frames deep whatever the pivots
do. A split that leaves nothing below the pivot has made no progress, and then,
only then, a second pass over the region peels the elements equal to the pivot
off it, which are in place already and end a region of one value. A region of
distinct keys costs one pass of the partition a level, and a region of 16 or
fewer is one insertion sort.

The cost is the partition's, once a level. On average there are `log n`
levels, so `O(n log n)` comparisons, and `O(n log n)` moves with the encoded
partition, a further `log n` factor with a rotation partition. A pivot that
splits badly adds levels, up to `n` of them, so the worst case is `n` passes
of the partition: `O(n^2)` moves with the encoded one.

Its catalog component is declared in `stable_quick.spec` and built by
`stable_quick.avbs`.
