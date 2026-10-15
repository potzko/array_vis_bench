# `pivot_last`

Pivot at the last index. Constant work, degenerates on sorted input. Quick sort subtracts it in its query; quick select and the partition kind keep it.

Catalog fragment `pivot_last.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `pkp_last` | `LastElement` | `last` | PartPivot |  |
