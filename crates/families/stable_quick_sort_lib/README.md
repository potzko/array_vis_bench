# `stable_quick_sort_lib`

Stable quick sort, `StableQuickSort<P, R>`: a quick sort whose split is a
stable three way partition `P`, the `StablePartition` axis, rotating with a
rotation `R`.

A quick sort is unstable because its partition throws an element across the
equal ones it passes. A stable partition keeps each side in order, so the
sort needs no tag and no index, only the scratch the partition and the
rotation share, allocated once. The pivot is a copied value, the median of
three values and past 128 elements the median of three such medians, so
nothing is moved to choose it. The region splits into below, equal and above;
the equal part is in place already, which is what lets a region of one value
end, and the sort recurses into the smaller of the other two parts and loops
on the larger, so the stack is `log n` frames deep whatever the pivots do. A
region of 16 or fewer is one insertion sort.

The cost is the partition's, once a level. On average there are `log n`
levels, so `O(n log n)` comparisons, and `O(n log n)` moves with the encoded
partition, a further `log n` factor with a rotation partition. A pivot that
splits badly adds levels, up to `n` of them, so the worst case is `n` passes
of the partition: `O(n^2)` moves with the encoded one.

The three way split costs two passes of the partition, the second over what
the first left at and above the pivot.

Its catalog component is declared in `stable_quick.spec` and built by
`stable_quick.avbs`.
