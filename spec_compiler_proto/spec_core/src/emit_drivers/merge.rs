//! `Category::Merge` driver.
//!
//! A standalone merge takes ONE array plus a split point `mid`: the two halves
//! `arr[..mid]` and `arr[mid..]` are each already sorted, and the merge
//! combines them in place, emitting the merge. The correctness battery checks
//! the merged output is sorted (and a permutation of the inputs).
//!
//! ABI surface assumed (the integrator adds it to `avb_abi`), verbatim:
//!   - `trait avb_abi::Merger { fn merge(arr: &mut [usize], mid: usize, logger: &mut dyn avb_abi::SortLogger<usize>); }`
//!   - `fn avb_abi::run_merge_with_input(input_name: &str, config: &avb_abi::RunConfig, merge_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>), logger: &mut dyn avb_abi::SortLogger<usize>)`
//!   - `fn avb_abi::assert_merges(merge_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>))`

use super::{DriverCode, DriverCtx};

pub fn driver(ctx: &DriverCtx) -> Result<DriverCode, String> {
    let abi = ctx.abi;
    // Both bodies need the same dyn-erased adapter from the type's `Merger`
    // impl to a plain fn pointer taking `(arr, mid, logger)`.
    let merge_dyn = format!(
        "fn merge_dyn(arr: &mut [usize], mid: usize, logger: &mut dyn {abi}::SortLogger<usize>) {{\n\
         \x20           <Ty as {abi}::Merger>::merge(arr, mid, logger);\n\
         \x20       }}"
    );
    Ok(DriverCode {
        run_default_body: format!(
            "{merge_dyn}\n\
             \x20       {abi}::run_merge_with_input(input_name, config, merge_dyn, logger);"
        ),
        run_correct_body: format!(
            "{merge_dyn}\n\
             \x20       {abi}::assert_merges(merge_dyn);"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_driver_renders_expected_bodies() {
        let code = driver(&DriverCtx { abi: "avb_abi" }).unwrap();
        assert!(code.run_default_body.contains("run_merge_with_input"));
        assert!(code.run_default_body.contains("<Ty as avb_abi::Merger>"));
        assert!(code.run_default_body.contains("merge_dyn"));
        assert!(code.run_correct_body.contains("assert_merges"));
    }
}
