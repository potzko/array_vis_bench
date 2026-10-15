# `circle_sort_lib`

Circle sort family: a recursive driver over the order of its three sub operations, a bottom up driver over a size traversal direction, a recursive shaker variant, and two finishing strategies.

Catalog fragment `circle.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `circle_recursive` | `CircleRecursiveOf<{order},{finish}>` | `circle sort<recursive: {order}, finish: {finish}>` | Sort | Sort |
| `circle_bottom_up` | `CircleBottomUpOf<{direction},{finish}>` | `circle sort<bottom-up: {direction}, finish: {finish}>` | Sort | Sort |
| `circle_shaker_recursive` | `CircleSortShakerRecursive` | `circle sort (recursive shaker)` | Sort | Sort |
| `circle_pre_order` | `PreOrder` | `pre-order` | CircleOrder |  |
| `circle_left_mid_right` | `LeftMidRight` | `left-mid-right` | CircleOrder |  |
| `circle_right_mid_left` | `RightMidLeft` | `right-mid-left` | CircleOrder |  |
| `circle_post_order` | `PostOrder` | `post-order` | CircleOrder |  |
| `circle_decreasing` | `Decreasing` | `decreasing` | CircleDirection |  |
| `circle_increasing` | `Increasing` | `increasing` | CircleDirection |  |
| `circle_shaker_dec_inc` | `ShakerDecInc` | `shaker dec->inc` | CircleDirection |  |
| `circle_shaker_inc_dec` | `ShakerIncDec` | `shaker inc->dec` | CircleDirection |  |
| `circle_converge` | `ConvergeFinish` | `converge` | CircleFinish |  |
| `circle_sc_insertion` | `InsertionShortCircuit` | `short circuit: insertion` | CircleFinish |  |

Query `circle.avbs` builds: `f`, `CircleRecursive`, `CircleBottomUp`, `CircleShaker`.
