# Sort Registry (Derive-Based)

This project uses a derive-only, auto-registration system to discover and run sorting algorithms without any code generation scripts.

## Overview

- Auto-registration: Each sort registers itself at startup via a `#[derive(SortRegistry)]` implementation using `ctor`.
- Standardized execution type: Sort closures are stored as `Arc<dyn Fn(&mut [usize], &mut NoOpLogger) + Send + Sync>`.
- Split responsibilities:
    - Metadata registry is in `sort_registry_core` (names, complexity, stability, category).
    - Runtime closure registry lives in `src/traits/mod.rs`.

## How It Works

1. Define your sort function generically: `fn sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U)`.
2. Use `create_sort!` to generate the trait impl and a monomorphic registration type:

```rust
create_sort!(
        my_sort_fn,
        "my_sort<partition: partition_style<pivot_selection: strategy>>",
        "O(N log N)",
        false
);
```

- `create_sort!` expands to:
    - `SortImp<T, U>` implementing `SortAlgo<T, U>`.
    - `SortReg` implementing `SortAlgo<usize, NoOpLogger>` and deriving `SortRegistry`.
    - The derive inserts the runtime closure into `SORT_REGISTRY` and registers metadata in `sort_registry_core`.

## APIs

- Metadata (from `sort_registry_core`):
    - `register_sort(name, big_o, stable, category)`
    - `get_registered_sorts() -> Vec<String>`

- Runtime closures (from `src/traits/mod.rs`):
    - `get_sort(name: &str) -> Option<Arc<dyn Fn(&mut [usize], &mut NoOpLogger) + Send + Sync>>`

## Usage Example

```rust
use crate::traits::{get_registered_sorts, get_sort, log_traits::NoOpLogger};

fn main() {
        // List all registered sort names
        let names = get_registered_sorts();
        println!("Registered sorts: {}", names.len());

        // Run a sort by name
        if let Some(run) = get_sort("quick_sort<partition: partition_left_right_pointers<pivot_selection: median_of_three>>") {
                let mut arr = vec![5usize, 3, 1, 4, 2];
                let mut logger = NoOpLogger;
                run(&mut arr, &mut logger);
                println!("Sorted: {:?}", arr);
        }
}
```

## Naming Convention

Many sorts use hierarchical, descriptive names to expose configuration:

- Example: `quick_sort<partition: partition_left_right_pointers<pivot_selection: median_of_three>>`
- This clarifies the partition strategy and pivot selection used.

## Adding New Sorts

1. Implement your generic sort function.
2. Call `create_sort!` with the display name, complexity, and stability.
3. The derive takes care of registration automatically—no scripts, no generated files.

## Notes

- The previous `generate_registration/` tooling and `generated_registrations.rs` file have been removed. All registration is now derive-based.
- The project standardizes on `usize` for data and `NoOpLogger` for monomorphic registrations.
