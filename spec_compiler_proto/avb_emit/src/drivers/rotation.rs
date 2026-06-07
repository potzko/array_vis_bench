//! `Category::Rotation` driver.
//!
//! A rotation takes an array + a split point `mid` and rotates the two blocks
//! `arr[..mid]` and `arr[mid..]` (a left-rotation by `mid`), emitting the
//! moves; its correctness battery checks the result equals the expected cyclic
//! shift. Modelled on the real `ROTATION_INPUTS` shape
//! (`generate -> (Vec<usize>, mid)`).
//!
//! ABI surface the emitted code targets, present in `avb_abi`:
//!   - trait avb_abi::Rotator { fn rotate(arr: &mut [usize], mid: usize, logger: &mut dyn avb_abi::SortLogger<usize>); }
//!   - fn avb_abi::run_rotation_with_input(input_name: &str, config: &avb_abi::RunConfig, rotate_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>), logger: &mut dyn avb_abi::SortLogger<usize>)
//!   - fn avb_abi::assert_rotations(rotate_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>))

use super::{DriverCode, DriverCtx};

pub fn driver(ctx: &DriverCtx) -> Result<DriverCode, String> {
    let abi = ctx.abi;
    // Both bodies need the same dyn-erased adapter from the type's `Rotator`
    // impl to a plain fn pointer.
    let rotate_dyn = format!(
        "fn rotate_dyn(arr: &mut [usize], mid: usize, logger: &mut dyn {abi}::SortLogger<usize>) {{\n\
         \x20           <Ty as {abi}::Rotator>::rotate(arr, mid, logger)\n\
         \x20       }}"
    );
    Ok(DriverCode {
        run_default_body: format!(
            "{rotate_dyn}\n\
             \x20       {abi}::run_rotation_with_input(input_name, config, rotate_dyn, logger);"
        ),
        run_correct_body: format!(
            "{rotate_dyn}\n\
             \x20       {abi}::assert_rotations(rotate_dyn);"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_driver_emits_expected_bodies() {
        let code = driver(&DriverCtx { abi: "avb_abi" }).unwrap();
        assert!(code.run_default_body.contains("run_rotation_with_input"));
        assert!(code.run_default_body.contains("<Ty as avb_abi::Rotator>"));
        assert!(code.run_default_body.contains("rotate_dyn"));
        assert!(code.run_correct_body.contains("assert_rotations"));
    }
}
