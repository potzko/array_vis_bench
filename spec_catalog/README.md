# `spec_catalog`

The unified catalog. Its build script gathers every `.spec` fragment and
`.avbs` query in its dependency closure, merges the fragments into one
registry, solves each query, and emits one registration module per ground
type. The default binary anchors this crate.

- `build.rs`: gather, merge, solve, emit, and write the golden counts.
- `kinds.avbs`: the first class kinds (rotations, small sorts, partitions, merges, scratchspace workflows).
- `consumers.avbs`: the consumer program run by `--consumers`.
- `src/lib.rs`: the emitted entries, the consumer runners, and the golden count and per family catalog tests.
- `catalog.golden`: every entry's category, name and inherited complexity, pinned by `catalog_identity_is_pinned`. Regenerate with `AVBS_BLESS=1` after an intended catalog change.
- `quick_port.spec`: a baked quick sort catalog kept only as the baseline for the slotted versus baked equivalence test.
