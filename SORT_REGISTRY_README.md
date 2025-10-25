# Sort Registry System

This project now includes a centralized sort registry that allows you to automatically discover and categorize all your sorting algorithms.

## How it Works

1. **Registry System**: The registry is defined in `src/traits/mod.rs` and provides:
   - `register_sort(name, big_o, stable, category)` - Register a sort
   - `get_registered_sorts()` - Get all registered sorts
   - `get_sorts_by_category(category)` - Get sorts by category
   - `init_sort_registry()` - Initialize the registry

2. **Automatic Generation**: The `generate_registration/` directory contains a script that automatically scans your codebase and generates registration code for all your sorts.

## Usage

### 1. Generate Registration Code

```bash
cd generate_registration
cargo run > ../generated_registrations.rs
```

This will create a file with a `register_all_sorts()` function that registers all your sorts.

### 2. Use the Registry

Add this to your `main.rs` or `lib.rs`:

```rust
use crate::traits::{init_sort_registry, register_sort, get_registered_sorts, get_sorts_by_category};

// Include the generated registrations
mod generated_registrations;
use generated_registrations::register_all_sorts;

fn main() {
    // Register all sorts
    register_all_sorts();
    
    // Get all sorts
    let all_sorts = get_registered_sorts();
    for sort in &all_sorts {
        println!("{} ({}) - Stable: {} - Category: {}", 
                 sort.name, sort.big_o, sort.stable, sort.category);
    }
    
    // Get sorts by category
    let bubble_sorts = get_sorts_by_category("bubble_sorts");
    let quick_sorts = get_sorts_by_category("quick_sorts");
    // etc.
}
```

### 3. Example Output

The registry provides information about each sort:
- **Name**: The sort's display name
- **Big O**: Time complexity
- **Stable**: Whether the sort is stable
- **Category**: The category (based on directory structure)

## Categories

The system automatically categorizes sorts based on their directory structure:
- `bubble_sorts/` → "bubble_sorts"
- `quick_sorts/` → "quick_sorts"
- `merge_sorts/` → "merge_sorts"
- `heap_sorts/` → "heap_sorts"
- etc.

## Benefits

1. **Centralized Discovery**: All sorts are automatically discovered and registered
2. **Categorization**: Sorts are automatically categorized by type
3. **Metadata**: Each sort includes name, complexity, and stability information
4. **Type Safety**: Fully type-safe and abstracted over data types and loggers
5. **Easy to Use**: Simple API for querying sorts

## Adding New Sorts

When you add a new sort:
1. Use the `create_sort!` macro as usual
2. Run the generation script to update registrations
3. The new sort will automatically be included in the registry

## Example Integration

```rust
// In your main.rs
use crate::traits::{init_sort_registry, get_registered_sorts, get_sorts_by_category};

fn main() {
    // Initialize and register all sorts
    init_sort_registry();
    register_all_sorts();
    
    // Use the registry
    let all_sorts = get_registered_sorts();
    println!("Found {} sorting algorithms", all_sorts.len());
    
    // Filter by category
    let stable_sorts: Vec<_> = all_sorts.iter()
        .filter(|s| s.stable)
        .collect();
    
    let fast_sorts: Vec<_> = all_sorts.iter()
        .filter(|s| s.big_o.contains("log"))
        .collect();
}
```

This system provides a clean, type-safe way to manage and discover all your sorting algorithms! 
