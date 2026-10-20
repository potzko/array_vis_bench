# `role`

Source of `array_vis_bench_traits`. The crate README one level up says what it is for.

- `auxiliary.rs`: `Auxiliary`, the scratchspace a merge uses for its working buffer.
- `direct_sort.rs`: `DirectSort`, the full sort role, and `direct_sort!`, its one line impl.
- `merge.rs`: `RotationMerge`, the merge role, and `run_rotation_merge`, its one-shot.
- `mod.rs`: Role traits, one per axis a leaf component can implement.
- `partition.rs`: `PartitionScheme`, partition algorithm role.
- `pivot.rs`: `PivotSelector` and `DualPivotSelector`, pivot-selection roles.
- `quick_select.rs`: `QuickSelect`, partition-based k-th-order-statistic finder role.
- `rotation.rs`: `Rotation`, rotation algorithm role + the shared in-place helpers every rotation impl reaches for.
- `scratchspace.rs`: `Scratchspace`, a set of contracts over one or more sequential lists, realised through a small auxiliary buffer.
- `small_sort.rs`: `SmallSort` and its companion traits.
