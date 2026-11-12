# `level_sort_lib`

Level sort, `LevelSort<P, R>`: a stable in place sort over a stable partition
`P` and a rotation `R`, the lazy refinement of the level ordered heap.

The build selects, by stable selection, the smallest `2^k - 1` elements of
the array at every power of two boundary from the largest down, so that
every level, the cells from `2^k - 1` to `2^(k + 1) - 1`, holds the next
rank interval, equal elements in original order within it and across the
levels: a level ordered heap in expected linear comparisons, the prefixes
halving. The sort then refines each
level by stable selection of its median, recursing into both halves until a
run of eight is left to the insertion sort. A stable selection is a
quickselect over the three way stable partition, the pivot a median of three
compared in place and never moved, stopping as soon as the wanted boundary
falls inside the pivot's equal group, which the stable split leaves in
original order; so equal elements never cross, and the sort is stable with
nothing but the rotation's scratch.

It is a stable quicksort with exact median splits by another route, with
the selection's constant: expected `O(n log n)` comparisons, `O(n log^2 n)`
moves by rotations, quadratic at worst as any quickselect. The levels are
the structure the sort refines; left unrefined, the first `2^k - 1` cells are
the `2^k - 1` smallest, ties to the earlier and equals in original order,
which is a stable top `k` in linear expected time. A partition keeps the
order within each of its classes, not across them, so a level is not in
original order as a whole, only among equals, which is all a stable sort
asks.

Its catalog component is declared in `level.spec` and built by `level.avbs`.
