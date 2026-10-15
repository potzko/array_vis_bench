# `rod_sort_lib`

Rod sort over a branching strategy and a rod merge. Hosts the BranchingStrategy trait, its six strategies and the two merges, since rod sort is their only consumer.

Catalog fragment `rod.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `rod_sort` | `RodSort<{strategy}, {merge}>` | `rod sort<strategy: {strategy}, merge: {merge}>` | Sort | Sort |
| `rod_classic` | `Classic` | `classic` | RodBranching |  |
| `rod_parity3` | `Parity3` | `3-parity` | RodBranching |  |
| `rod_log_parity` | `LogParity` | `log-parity` | RodBranching |  |
| `rod_root_parity` | `RootParity` | `root-parity` | RodBranching |  |
| `rod_optimised` | `Optimised` | `optimised` | RodBranching |  |
| `rod_fibonacci` | `Fibonacci` | `fibonacci` | RodBranching |  |
| `rod_insertion` | `InsertionMerge` | `insertion` | RodMerge |  |
| `rod_aux` | `AuxMerge` | `aux` | RodMerge |  |

Query `rod.avbs` builds: `RodSorts`.
