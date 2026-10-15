# `comb_sort_lib`

Comb sort family: the driver and the const generic shrink ratio type, with six ratios declared as components.

Catalog fragment `comb.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `comb_sort` | `CombSortOf<{ratio}>` | `comb sort<{ratio}>` | Sort | Sort |
| `comb_13` | `CombSortRatio<10, 13>` | `1.3` | CombRatio |  |
| `comb_sqrt2` | `CombSortRatio<70, 99>` | `sqrt2 ~ 1.414` | CombRatio |  |
| `comb_phi` | `CombSortRatio<55, 89>` | `phi ~ 1.618` | CombRatio |  |
| `comb_four_thirds` | `CombSortRatio<3, 4>` | `4/3` | CombRatio |  |
| `comb_eleven_eighths` | `CombSortRatio<8, 11>` | `11/8` | CombRatio |  |
| `comb_five_fourths` | `CombSortRatio<4, 5>` | `5/4` | CombRatio |  |

Query `comb.avbs` builds: `CombSorts`.
