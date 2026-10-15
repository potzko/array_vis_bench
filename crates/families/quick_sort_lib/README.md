# `quick_sort_lib`

Quick sort family: the canonical and deferred drivers over one self pivoting partition and a small sort, the Pivoted and DualPivoted composites that pair a scheme with a selector, the Yaroslavskiy dual pivot partition, and the combined and ninther dual selectors. The composites are also the first class partition kind.

Catalog fragment `quick.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `combined` | `CombinedSelector<{a}, {b}>` | `combined<{a}, {b}>` | PartPivotDual |  |
| `ninther_dual` | `NintherDualPivot` | `ninther` | PartPivotDual |  |
| `pks_dual` | `DualPivotPartition` | `dual` | PartSchemeDual |  |
| `pk_single` | `Pivoted<{scheme}, {pivot}>` | `{scheme}<{pivot}>` | PartitionKind SelectPartition | Partition |
| `pk_dual` | `DualPivoted<{pivot}>` | `dual<{pivot}>` | PartitionKind SelectPartition | Partition |
| `quick_sort` | `CanonicalQuickSort<{part}, {small_sort}>` | `quick sort<part: {part}, small: {small_sort}>` | Sort | Sort |
| `deferred_quick_sort` | `DeferredCanonicalQuickSort<{part}, {small_sort}>` | `deferred quick sort<part: {part}, small: {small_sort}>` | Sort | Sort |
| `beap_dps_first` | `CombinedSelector<FirstElement, FirstElement>` | `first` | BeapDualPivot |  |
| `beap_dps_middle` | `CombinedSelector<MiddleElement, MiddleElement>` | `middle` | BeapDualPivot |  |
| `beap_dps_ninther` | `NintherDualPivot` | `ninther 1/3 + 2/3` | BeapDualPivot |  |

Query `quick.avbs` builds: `QuickSorts`, `DeferredQuickSorts`.
