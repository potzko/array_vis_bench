//! Metadata-only leaf — see Cargo.toml for the role declarations.
//!
//! Heap-internal components are deeply coupled to the
//! `Heap`/`Compare`/`HeapLayout` machinery in `array_vis_bench`, so the
//! types stay in their original modules. The wiring crate already
//! imports them by their unqualified names through the family! `uses`
//! blocks, so the dep-graph scanner just needs the metadata to land
//! somewhere it can find — that's this crate's only job.
