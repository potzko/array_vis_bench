//! Registry, harness, inputs, and visualiser shell for `array_vis_bench`.
//!
//! Wiring crates (`array_vis_bench`, `array_vis_bench_min`, …) depend on
//! this crate plus whichever algorithm leaves they want in scope. The
//! algorithm leaves register their entries into the distributed slices
//! declared here (`ALGORITHMS`, `SORT_INPUTS`, …); this crate is
//! intentionally algorithm-free so the per-leaf compile-time win
//! propagates all the way through to the binary.

pub mod array_gen;
pub mod bench_registry;
pub mod inputs;
/// The `Main` consumer abstraction (visualise / correctness / benchmark over a
/// set of registered algorithms) — the Rust counterpart of AVBS `consumers`.
pub mod mains;
/// Per-kind correctness batteries as a trait (`CorrectnessSuite`), one impl per
/// first-class algorithm kind — the seam the spec emit dispatches each leaf's
/// `run_correctness` through.
pub mod suites;
pub mod visualise;
