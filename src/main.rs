use array_vis_bench::traits::get_registered_sorts;
use array_vis_bench::traits::log_traits::{SortLog, VisualizerLogger};
use array_vis_bench::visualise::visualise_sort;
use array_vis_bench::utils::array_gen::{get_rand_arr, get_rand_arr_in_range, get_arr, get_reversed_arr};
use std::io::{self, Write};

fn main() {
    // Sorts are auto-registered via derive macros at startup
    
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
    print!("\nEnter array size (10-1000): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let size: usize = input.trim().parse().unwrap_or(50);
    let size = size.max(10).min(1000); // Clamp between 10 and 1000
    
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
    // Map sort names to their category and implementation
    // First, handle hierarchical quick sort names generically
    if let Some(choice) = parse_quick_sort_name(sort_name) {
        return choice;
    }
    match sort_name {
        // Merge sorts
        "merge sort" => vec!["merge_sorts".to_string(), "classic_merge_sort".to_string()],
        "merge sort bottom up" => vec!["merge_sorts".to_string(), "merge_sort_bottom_up".to_string()],
        "merge sort optimized" => vec!["merge_sorts".to_string(), "merge_sort_optimized".to_string()],
        "merge sort bottom up optimized" => vec!["merge_sorts".to_string(), "merge_sort_bottom_up_optimized".to_string()],
        "merge sort outside lists" => vec!["merge_sorts".to_string(), "merge_sort_outside_lists".to_string()],
        "rotate merge sort" => vec!["merge_sorts".to_string(), "rotate_merge_sort".to_string()],
        "rotate merge sort optimized" => vec!["merge_sorts".to_string(), "rotate_merge_sort_optimized".to_string()],
        "rotate merge sort bottom up" => vec!["merge_sorts".to_string(), "rotate_merge_sort_bottom_up".to_string()],
        "rotate merge sort bottom up optimized" => vec!["merge_sorts".to_string(), "rotate_merge_sort_bottom_up_optimized".to_string()],
        
        // Quick sorts (legacy explicit names)
        "quick_sort<partition: partition_left_right_pointers<pivot_selection: median_of_three>>" => vec!["quick_sorts".to_string(), "midian_pivot_quick_sort".to_string()],
        "quick_sort<mode: iterative>" => vec!["quick_sorts".to_string(), "iterative_quick_sort".to_string()],
        
        // Heap sorts
        "heap sort classic" => vec!["heap_sort".to_string(), "classic_heap_sorts".to_string(), "heap_sort_classic".to_string()],
        "heap sort base 3" => vec!["heap_sort".to_string(), "classic_heap_sorts".to_string(), "base_3_heap".to_string()],
        "heap sort base 16" => vec!["heap_sort".to_string(), "classic_heap_sorts".to_string(), "base_16_heap".to_string()],
        "heap sort base 256" => vec!["heap_sort".to_string(), "classic_heap_sorts".to_string(), "base_256_heap".to_string()],
        "heap quick sort" => vec!["heap_sort".to_string(), "heap_quick_sort".to_string()],
        "heap quick sort optimized" => vec!["heap_sort".to_string(), "heap_quick_sort_optimized".to_string()],
        "heap quick sort optimized tmp" => vec!["heap_sort".to_string(), "heap_quick_sort_optimized_tmp".to_string()],
        "weak heap sort" => vec!["heap_sort".to_string(), "weak_heap_sort".to_string()],
        
        // Bubble sorts
        "bubble sort" => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()],
        "bubble sort recursive" => vec!["bubble_sorts".to_string(), "bubble_sort_recursive".to_string()],
        "shaker sort" => vec!["bubble_sorts".to_string(), "shaker_sort".to_string()],
        "odd-even bubble sort" => vec!["bubble_sorts".to_string(), "odd_even_bubble_sort".to_string()],
        
        // Shell sorts
        "shell sort classic" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_classic".to_string()],
        "shell sort knuth" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_knuth".to_string()],
        "shell sort hibbard" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_hibbard".to_string()],
        "shell sort sedgewick" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_sedgewick".to_string()],
        "shell sort sedgewick branching" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_sedgewick_branching".to_string()],
        "shell sort sedgewick ordered insertion" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_sedgewick_ordered_insertion".to_string()],
        "shell sort classic ordered insertion" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_classic_ordered_insertion".to_string()],
        "shell sort classic dissonance" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_classic_dissonance".to_string()],
        "shell sort optimized 256 elements" => vec!["shell_sorts".to_string(), "classic_shell_sorts".to_string(), "shell_optimized_256_elements".to_string()],
        "shell shell sort classic" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_classic".to_string()],
        "shell shell sort optimised" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_optimised".to_string()],
        "shell shell sort log parity" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_log_parity".to_string()],
        "shell shell sort root parity" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_root_parity".to_string()],
        "shell shell sort 3 parity" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_3parity".to_string()],
        "shell shell sort fibonacci" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_fibonacci".to_string()],
        "shell sort 2.25 shrink factor" => vec!["shell_sorts".to_string(), "shell_shell_sorts".to_string(), "shell_shell_sort_merge_variation_2_parity".to_string()],
        
        // Circle sorts
        "circle sort recursive" => vec!["circle_sorts".to_string(), "circle_sort_recursive".to_string()],
        "circle sort recursive optimised" => vec!["circle_sorts".to_string(), "circle_sort_recursive_optimised".to_string()],
        "circle sort recursive increasing size" => vec!["circle_sorts".to_string(), "circle_sort_recursive_increasing_size".to_string()],
        "circle sort bottom up" => vec!["circle_sorts".to_string(), "circle_sort_bottom_up".to_string()],
        "circle sort bottom up optimised" => vec!["circle_sorts".to_string(), "circle_sort_bottom_up_optimised".to_string()],
        "circle sort bottom up increasing size" => vec!["circle_sorts".to_string(), "circle_sort_bottom_up_increasing_size".to_string()],
        "shaker circle sort" => vec!["circle_sorts".to_string(), "shaker_circle_sort".to_string()],
        "shaker circle sort bottom up" => vec!["circle_sorts".to_string(), "shaker_circle_sort_bottom_up".to_string()],
        "shaker circle sort bottom up b" => vec!["circle_sorts".to_string(), "shaker_circle_sort_bottom_up_b".to_string()],
        "stooge circle sort" => vec!["circle_sorts".to_string(), "stooge_circle_sort".to_string()],
        "stooge circle sort reversed" => vec!["circle_sorts".to_string(), "stooge_circle_sort_reversed".to_string()],
        
        // Other sorts
        "insertion sort" => vec!["insertion_sorts".to_string(), "insertion_sort".to_string()],
        "cycle sort" => vec!["cycle_sorts".to_string(), "cycle_sort".to_string()],
        "comb sort classic" => vec!["comb_sorts".to_string(), "comb_classic".to_string()],
        "comb sort random gaps" => vec!["comb_sorts".to_string(), "comb_random_gaps".to_string()],
        "slow sort" => vec!["fun_sorts".to_string(), "slow_sort".to_string()],
        "stooge sort" => vec!["fun_sorts".to_string(), "stooge_sort".to_string()],
        "bad heap sort" => vec!["fun_sorts".to_string(), "bad_heap_sort".to_string()],
        "bad heap sort alt" => vec!["fun_sorts".to_string(), "bad_heap_sort_alt".to_string()],
        "cyclent sort" => vec!["fun_sorts".to_string(), "cyclent_sort".to_string()],
        "cyclent sort stack" => vec!["fun_sorts".to_string(), "cyclent_sort_stack".to_string()],
        "cyclent sort stack optimized" => vec!["fun_sorts".to_string(), "cyclent_sort_stack_optimized".to_string()],
        "quick surrender" => vec!["fun_sorts".to_string(), "quick_surrender".to_string()],
        "quick surrender optimised" => vec!["fun_sorts".to_string(), "quick_surrender_optimised".to_string()],
        "shell sort hibbard jumps" => vec!["fun_sorts".to_string(), "random_shell_sort".to_string()],
        
        // Default fallback
        _ => {
            println!("⚠️  Warning: Sort '{}' not found in mapping, using default heap sort", sort_name);
            vec!["heap_sort".to_string(), "heap_quick_sort_optimized".to_string()]
        }
    }
}

fn parse_quick_sort_name(sort_name: &str) -> Option<Vec<String>> {
    // Supports names like:
    // quick_sort<partition: partition_left_left<pivot_selection: last_element>>
    // quick_sort_optimized<partition: partition_left_right_pointers<pivot_selection: median_of_three>>
    // quick_sort<mode: iterative>
    let trimmed = sort_name.trim();
    let is_opt = if let Some(rest) = trimmed.strip_prefix("quick_sort_optimized<") {
        Some(rest)
    } else {
        None
    };
    let is_std = if let Some(rest) = trimmed.strip_prefix("quick_sort<") {
        Some(rest)
    } else {
        None
    };

    // Mode-only variant
    if trimmed == "quick_sort<mode: iterative>" {
        return Some(vec!["quick_sorts".to_string(), "iterative_quick_sort".to_string()]);
    }

    let inner = match (is_opt, is_std) {
        (Some(rest), None) => (true, rest),
        (None, Some(rest)) => (false, rest),
        _ => return None,
    };

    let (optimized, rest) = inner;
    // Expect: partition: X<pivot_selection: Y>>
    let rest = rest.strip_suffix('>')?; // remove trailing '>'
    // Split on first '<' to separate partition and pivot_selection
    let mut parts = rest.splitn(2, '<');
    let partition_part = parts.next()?; // "partition: partition_left_left"
    let pivot_part = parts.next()?; // "pivot_selection: last_element"

    // Extract partition strategy
    let partition = partition_part.trim();
    let partition = partition.strip_prefix("partition:")?.trim();

    // Extract pivot selection
    let pivot = pivot_part.strip_suffix('>') // in case of nested '>>' or malformed
        .unwrap_or(pivot_part)
        .trim();
    let pivot = pivot.strip_prefix("pivot_selection:")?.trim();

    // Special-case moving pivot (dedicated implementation)
    if partition == "partition_left_right_pointers" && pivot == "moving_pivot_median_of_three" {
        return Some(vec!["quick_sorts".to_string(), "quick_sort_left_right_pointers_moving_pivot".to_string()]);
    }

    // Map to available implementations (generic runner)
    let choice = route_quick_sort_combo(optimized, partition, pivot);
    Some(choice)
}

fn route_quick_sort_combo(optimized: bool, partition: &str, pivot: &str) -> Vec<String> {
    // Route all combinations to a generic quick sort runner with config
    vec![
        "quick_sorts".to_string(),
        "generic_quick_sort".to_string(),
        partition.to_string(),
        pivot.to_string(),
        optimized.to_string(),
    ]
}