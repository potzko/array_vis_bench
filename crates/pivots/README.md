# pivots

One crate per pivot selector. Each implements `PivotSelector` and declares
the same type under up to three roles: `PartPivot` for the shared partition
system, `BeapPiv` for beap sort and `HeapPiv` for the heap quick build.

- `pivot_first`: index zero.
- `pivot_last`: the last index.
- `pivot_middle`: the midpoint.
- `pivot_median3`: median of first, middle and last.
- `pivot_median_of_medians`: linear time guaranteed split.
- `pivot_ninther`: median of three medians of three.
