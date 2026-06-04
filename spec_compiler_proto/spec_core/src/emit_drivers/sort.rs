//! `Category::Sort` driver. A sort runs on a single array; the input comes from
//! the matching `SORT_INPUTS` entry (here synthesised from `RunConfig`). The
//! correctness battery is the shared sort battery.

use super::{DriverCode, DriverCtx};

pub fn driver(ctx: &DriverCtx) -> DriverCode {
    let abi = ctx.abi;
    // Both bodies need the same dyn-erased adapter from the type's `SortAlgo`
    // impl to a plain fn pointer.
    let sort_dyn = format!(
        "fn sort_dyn(arr: &mut [usize], logger: &mut dyn {abi}::SortLogger<usize>) {{\n\
         \x20           <Ty as {abi}::SortAlgo<usize, dyn {abi}::SortLogger<usize>>>::sort(arr, logger);\n\
         \x20       }}"
    );
    DriverCode {
        run_default_body: format!(
            "{sort_dyn}\n\
             \x20       {abi}::run_sort_with_input(input_name, config, sort_dyn, logger);"
        ),
        run_correct_body: format!(
            "{sort_dyn}\n\
             \x20       {abi}::assert_sorts(sort_dyn);"
        ),
    }
}
