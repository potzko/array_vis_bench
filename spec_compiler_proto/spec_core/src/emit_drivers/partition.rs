//! `Category::Partition` driver.
//!
//! A partition isn't run like a whole sort: its `run_with_input` takes a pivot
//! input, partitions the array around it, and emits the scan; its correctness
//! battery checks the partition invariant (everything left of the returned
//! index ≤ pivot ≤ everything right). Model it on the real
//! `crates/registries/quick_partition_registry/src/lib.rs` `run_with_input`.

use super::{DriverCode, DriverCtx};

pub fn driver(ctx: &DriverCtx) -> Result<DriverCode, String> {
    // ABI surface assumed by the emitted code (the INTEGRATOR adds these to
    // `avb_abi`):
    //   - trait avb_abi::Partitioner { fn partition(arr: &mut [usize], pivot_index: usize, logger: &mut dyn avb_abi::SortLogger<usize>) -> usize; }
    //   - fn avb_abi::run_partition_with_input(input_name: &str, config: &avb_abi::RunConfig, partition_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>) -> usize, logger: &mut dyn avb_abi::SortLogger<usize>)
    //   - fn avb_abi::assert_partitions(partition_fn: fn(&mut [usize], usize, &mut dyn avb_abi::SortLogger<usize>) -> usize)
    let abi = ctx.abi;
    // Both bodies need the same dyn-erased adapter from the type's `Partitioner`
    // impl to a plain fn pointer.
    let part_dyn = format!(
        "fn part_dyn(arr: &mut [usize], pivot_index: usize, logger: &mut dyn {abi}::SortLogger<usize>) -> usize {{\n\
         \x20           <Ty as {abi}::Partitioner>::partition(arr, pivot_index, logger)\n\
         \x20       }}"
    );
    Ok(DriverCode {
        run_default_body: format!(
            "{part_dyn}\n\
             \x20       {abi}::run_partition_with_input(input_name, config, part_dyn, logger);"
        ),
        run_correct_body: format!(
            "{part_dyn}\n\
             \x20       {abi}::assert_partitions(part_dyn);"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_partition_bodies() {
        let code = driver(&DriverCtx { abi: "avb_abi" }).unwrap();
        assert!(code.run_default_body.contains("run_partition_with_input"));
        assert!(code.run_default_body.contains("<Ty as avb_abi::Partitioner>"));
        assert!(code.run_default_body.contains("part_dyn"));
        assert!(code.run_correct_body.contains("assert_partitions"));
    }
}
