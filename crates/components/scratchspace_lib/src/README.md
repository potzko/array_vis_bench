# `src`

Source of `scratchspace_lib`. The crate README one level up says what it is for.

- `bufferless.rs`: `bufferless_merge`, the stable in place merge by rotations that the bufferless backings share.
- `buffers.rs`: the block merge buffers, `InArrayBuffer` and `FixedBuffer`.
- `keys.rs`: the `Keys` backings, `InArrayKeys` and `NoKeys`.
- `lib.rs`: Auxiliary (scratchspace) backings + the merge that consumes them.
- `workflows.rs`: Scratchspace workflows (runnable routines over the ops) + a scratchspace -backed sort.
