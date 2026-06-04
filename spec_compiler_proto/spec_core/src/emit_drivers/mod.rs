//! Per-category emit drivers. Each `Category` emits a different `run_with_input`
//! body + correctness battery (a Sort runs on an array; a Partition takes a
//! pivot and emits its scan; a Merge takes two runs; …). One file per category
//! keeps these **disjoint** so they can be filled in by parallel agents — see
//! `docs/todo.md` Phase 0.4 / "Parallelization". The dispatch + the module
//! declarations below are owned by the integrator; a fleet task edits only its
//! own `<category>.rs` (and adds its stub algorithm + a test).

pub mod merge;
pub mod partition;
pub mod rotation;
pub mod sort;

/// What a driver needs to render its fn bodies: the ABI crate path (`avb_abi`
/// here, `array_vis_bench_core::bench_registry` / `…_traits` in the real repo).
/// The emitted code always names the algorithm type as the local alias `Ty`.
pub struct DriverCtx<'a> {
    pub abi: &'a str,
}

/// The category-specific fn bodies the entry template splices in.
pub struct DriverCode {
    pub run_default_body: String,
    pub run_correct_body: String,
}

/// Pick the driver for a category. Adding a category = implement its file and
/// add one arm here.
pub fn driver(category: &str, ctx: &DriverCtx) -> Result<DriverCode, String> {
    match category {
        "Sort" => Ok(sort::driver(ctx)),
        "Partition" => partition::driver(ctx),
        "Merge" => merge::driver(ctx),
        "Rotation" => rotation::driver(ctx),
        other => Err(format!(
            "no emit driver for category `{other}` — add `spec_core/src/emit_drivers/\
             {}.rs` and an arm in `driver()`",
            other.to_lowercase()
        )),
    }
}
