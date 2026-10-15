# `merge_sort_lib`

Merge sort family: top down, bottom up, naive and natural classic merge sorts, the rotation merge sorts, the RotationMerge trait with its naive and smaller side merges, and the auxiliary merge the scratchspace backings plug into. The merge kinds the catalog emits as first class merges are declared here as well.

Catalog fragment `merge.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `merge_top_down` | `TopDownMergeSort<{small_sort}, {ping_pong}, {early_exit}>` | `merge sort<small: {small_sort}, pp={ping_pong}, ee={early_exit}>` | Sort | Sort |
| `merge_bottom_up` | `BottomUpMergeSort<{small_sort}, {ping_pong}, {early_exit}>` | `bottom-up merge sort<small: {small_sort}, pp={ping_pong}, ee={early_exit}>` | Sort | Sort |
| `merge_naive` | `NaiveMergeSort<{small_sort}>` | `naive merge sort<small: {small_sort}>` | Sort | Sort |
| `merge_natural` | `NaturalMergeSort<{ping_pong}, {early_exit}>` | `natural merge sort<pp={ping_pong}, ee={early_exit}>` | Sort | Sort |
| `merge_rot_top_down` | `TopDownRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>` | `rotation merge sort<small: {small_sort}, merge: {rmerge}>` | Sort | Sort |
| `merge_rot_bottom_up` | `BottomUpRotationMergeSort<{small_sort}, {rmerge}, {early_exit}>` | `bottom-up rotation merge sort<small: {small_sort}, merge: {rmerge}>` | Sort | Sort |
| `mrm_naive` | `NaiveRotationMerge<{rot}>` | `naive<{rot}>` | RotationMerge |  |
| `mrm_smaller_side` | `SmallerSideRotationMerge<{rot}>` | `smaller-side<{rot}>` | RotationMerge |  |
| `mk_naive` | `NaiveRotationMerge<{rot}>` | `naive<{rot}>` | MergeKind | Merge |
| `mk_smaller` | `SmallerSideRotationMerge<{rot}>` | `smaller-side<{rot}>` | MergeKind | Merge |

Query `merge.avbs` builds: `TopDownMergeSorts`, `BottomUpMergeSorts`, `NaiveMergeSorts`, `NaturalMergeSorts`, `TopDownRotationMergeSorts`, `BottomUpRotationMergeSorts`.
