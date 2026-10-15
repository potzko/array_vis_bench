# `scratchspace_lib`

Scratchspace backings and what is built on them: three Auxiliary backings behind AuxiliaryMerge, three Scratchspace backings, four scripted workflows that net to identity, and the nightmare natural merge sort that runs entirely on the scratchspace contract.

Catalog fragment `scratchspace.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `aux_full` | `FullBuffer` | `full` | Auxiliary |  |
| `aux_half` | `HalfBuffer` | `half` | Auxiliary |  |
| `aux_none` | `NoBuffer` | `none` | Auxiliary |  |
| `mk_aux` | `AuxiliaryMerge<{aux}>` | `auxiliary<{aux}>` | MergeKind | Merge |
| `bk_full` | `FullScratch` | `full` | Scratchspace |  |
| `bk_sqrt` | `SqrtScratch` | `sqrt` | Scratchspace |  |
| `bk_none` | `NoScratch` | `none` | Scratchspace |  |
| `wf_exercise` | `Exercise<{backing}>` | `exercise<{backing}>` | ScratchspaceKind | Scratchspace |
| `wf_park_sweep` | `ParkSweep<{backing}>` | `park-sweep<{backing}>` | ScratchspaceKind | Scratchspace |
| `wf_park_loop` | `ParkLoop<{backing}>` | `park-loop<{backing}>` | ScratchspaceKind | Scratchspace |
| `wf_relocate` | `Relocations<{backing}>` | `relocate<{backing}>` | ScratchspaceKind | Scratchspace |
| `ss_nightmare` | `ScratchspaceMergeSort<{backing}>` | `scratchspace-nightmare<{backing}>` | Sort | Sort |

Query `scratchspace.avbs` builds: `Nightmares`.
