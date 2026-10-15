# `beap_sort_lib`

Beap sort: a bi-parental heap in triangular layout plugged into the heap sort driver from heap_sort_lib. Three drivers, classic build, quick select build and dual pivot build, each over heap direction and deep heapify strategy.

Catalog fragment `beap.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `beap_sort_classic` | `BeapSortClassicOf<{d}, {dh}>` | `beap sort<dir: {d}, build: {dh}>` | Sort | Sort |
| `beap_sort_quick_build` | `BeapSortQuickOf<{d}, {dh}>` | `beap sort quick build<dir: {d}, build: {dh}>` | Sort | Sort |
| `beap_sort_dual_build` | `BeapSortDualOf<{d}, {dh}>` | `beap sort quick build dual pivot<dir: {d}, build: {dh}>` | Sort | Sort |

Query `beap.avbs` builds: `BeapClassic`, `BeapQuickBuild`, `BeapQuickBuildDual`.
