# `quick_select_lib`

Recursive and iterative quick select over one self pivoting partition scheme. The partition population is curated in the query.

Catalog fragment `quick_select.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `qs_recursive` | `RecursiveQuickSelect<{part}>` | `quick select: recursive<{part}>` | QuickSelect | QuickSelect |
| `qs_iterative` | `IterativeQuickSelect<{part}>` | `quick select: iterative<{part}>` | QuickSelect | QuickSelect |

Query `quick_select.avbs` builds: `RecursiveSelectPiv`, `IterativeSelectPiv`, `he`, `RecursiveSelectHe`, `IterativeSelectHe`.
