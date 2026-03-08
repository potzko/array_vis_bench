use array_vis_bench::traits::get_registered_sorts;
use array_vis_bench::traits::log_traits::{SortLog, VisualizerLogger};
use array_vis_bench::visualise::visualise_sort;
use array_vis_bench::utils::array_gen::{get_rand_arr, get_rand_arr_in_range, get_arr, get_reversed_arr};
use std::io::{self, Write};

fn main() {
    // Verify that every registered sort has a visualization dispatch route.
    // This panics at startup — before any user interaction — if a sort was
    // added to the registry but create_sort_choice() doesn't know how to
    // route it.
    validate_sort_routing();

    println!("Array Visualization Benchmark");
    println!("==============================");
    
    // Step 1: Select sorting algorithm
    let registered_sorts = get_registered_sorts();
    println!("\nAvailable Sorting Algorithms:");
    for (i, sort_name) in registered_sorts.iter().enumerate() {
        println!("  {}: {}", i + 1, sort_name);
    }
    
    let selected_sort = get_user_selection("Select a sorting algorithm", 1, registered_sorts.len());
    let sort_name = &registered_sorts[selected_sort - 1];
    println!("Selected: {}", sort_name);
    
    // Step 2: Select array type
    println!("\nArray Types:");
    println!("  1: Random");
    println!("  2: Ascending (0, 1, 2, ...)");
    println!("  3: Descending (n-1, n-2, ..., 0)");
    println!("  4: Random in range (0 to size-1)");
    
    let array_type = get_user_selection("Select array type", 1, 4);
    
    // Step 3: Select array size
    println!("\nEnter array size");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let size: usize = input.trim().parse().unwrap_or(500);
    
    println!("Array size: {}", size);
    
    // Generate array based on user selection
    let mut arr = match array_type {
        1 => {
            println!("Generating random array...");
            get_rand_arr(size)
        },
        2 => {
            println!("Generating ascending array...");
            get_arr(size)
        },
        3 => {
            println!("Generating descending array...");
            get_reversed_arr(size)
        },
        4 => {
            println!("Generating random array in range...");
            get_rand_arr_in_range(size, 0, size)
        },
        _ => {
            println!("Defaulting to random array...");
            get_rand_arr(size)
        }
    };
    
    println!("Original array (first 20 elements): {:?}", 
             if arr.len() > 20 { &arr[..20] } else { &arr });
    if arr.len() > 20 {
        println!("   ... and {} more elements", arr.len() - 20);
    }
    
    // Create visualizer logger
    let mut logger = VisualizerLogger {
        log: Vec::<SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    
    // Create sort selection format that the system expects
    let sort_choice = create_sort_choice(sort_name);
    
    println!("\nGenerating visualization...");
    
    // Use the existing visualization system to create a GIF
    visualise_sort(&mut arr, &mut logger, &sort_choice);
    
    println!("Sorted array (first 20 elements): {:?}", 
             if arr.len() > 20 { &arr[..20] } else { &arr });
    if arr.len() > 20 {
        println!("   ... and {} more elements", arr.len() - 20);
    }
    
    println!("Statistics:");
    println!("  - Array size: {}", arr.len());
    println!("  - Operations logged: {}", logger.log.len());
    println!("  - GIF saved as: output.gif");
    
    println!("\nVisualization complete!");
}

fn get_user_selection(prompt: &str, min: usize, max: usize) -> usize {
    loop {
        print!("\n{} ({} to {}): ", prompt, min, max);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    println!("Error: Please enter a number.");
                    continue;
                }
                
                match trimmed.parse::<usize>() {
                    Ok(selection) => {
                        if selection >= min && selection <= max {
                            return selection;
                        } else {
                            println!("Error: Number {} is out of range. Please enter a number between {} and {}.", selection, min, max);
                        }
                    },
                    Err(_) => {
                        println!("Error: '{}' is not a valid number. Please enter a number between {} and {}.", trimmed, min, max);
                    }
                }
            },
            Err(_) => {
                println!("Error: Error reading input. Please try again.");
                continue;
            }
        }
    }
}

fn create_sort_choice(sort_name: &str) -> Vec<String> {
    // Shell sorts — checked against GAP_SEQUENCES so new variants are
    // automatically routable without touching this function.
    if let Some(choice) = array_vis_bench::sorts::shell_sorts::sort_choice(sort_name) {
        return choice;
    }

    match sort_name {
        "insertion sort" => vec!["insertion_sorts".to_string(), "insertion_sort".to_string()],

        // When a new sort family is reconnected (see REFACTOR_PLAN.md), add
        // its arm here — or, better, expose a sort_choice() function from that
        // module (like shell_sorts::sort_choice) so this function never needs
        // to change.
        _ => panic!(
            "\n\
             ┌─────────────────────────────────────────────────────────┐\n\
             │              SORT DISPATCH BUG DETECTED                 │\n\
             └─────────────────────────────────────────────────────────┘\n\
             Sort '{}' is registered in SORT_REGISTRY but has no\n\
             visualization dispatch route in create_sort_choice().\n\
             \n\
             To fix: add a route for this sort family in main.rs, or\n\
             expose a sort_choice() function from that sort's module\n\
             (see shell_sorts::sort_choice for the pattern to follow).\n",
            sort_name
        ),
    }
}

/// Panics at startup if any registered sort lacks a visualization route.
///
/// Runs before any user interaction so the bug is caught immediately on
/// launch, not silently mid-session.
fn validate_sort_routing() {
    for sort_name in get_registered_sorts() {
        // This will panic with a clear message if the route is missing.
        create_sort_choice(&sort_name);
    }
}

// Quick sort name parsing removed — will be reimplemented in Phase 2.
// See REFACTOR_PLAN.md.