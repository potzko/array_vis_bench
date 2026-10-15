# `shell_sort_lib`

Shell sort and ordered shell sort over a gap sequence. Hosts the GapSequence trait and nine sequences.

Catalog fragment `shell.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `shell_sort` | `ShellSort<{seq}>` | `shell sort<sequence: {seq}>` | Sort | Sort |
| `shell_sort_ordered` | `ShellSortOrdered<{seq}>` | `shell sort ordered<sequence: {seq}>` | Sort | Sort |
| `classic` | `Classic` | `classic` | GapSequence |  |
| `knuth` | `Knuth` | `knuth` | GapSequence |  |
| `hibbard` | `Hibbard` | `hibbard` | GapSequence |  |
| `sedgewick` | `Sedgewick` | `sedgewick` | GapSequence |  |
| `sedgewick_branching` | `SedgewickBranching` | `sedgewick-branching` | GapSequence |  |
| `ciura` | `Ciura` | `ciura` | GapSequence |  |
| `tokuda` | `Tokuda` | `tokuda` | GapSequence |  |
| `pratt` | `Pratt` | `pratt` | GapSequence |  |
| `optimized256` | `Optimized256` | `optimized-256` | GapSequence |  |

Query `shell.avbs` builds: `ShellSorts`, `ShellSortsOrdered`.
