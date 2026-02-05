use crate::traits::SortLogger;
use crate::traits::sort_traits::SortAlgo;

pub fn run<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U, choice: &[String]) -> Vec<String> {
    // Expect choice = ["generic_quick_sort", partition, pivot, optimized?]
    if choice.len() < 3 {
        logger.mark_mssg("generic_quick_sort: invalid configuration");
        return vec!["name: generic_quick_sort<invalid>".to_string()];
    }
    let partition = choice[1].as_str();
    let pivot = choice[2].as_str();
    let optimized = choice.get(3).map(|s| s == "true").unwrap_or(false);

    let name = format!(
        "quick_sort{}<partition: {}<pivot_selection: {}>>",
        if optimized { "_optimized" } else { "" },
        partition,
        pivot
    );

    quick_sort_recursive(arr, logger, partition, pivot, optimized);

    vec![format!("name: {}", name)]
}

fn quick_sort_recursive<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    partition: &str,
    pivot: &str,
    optimized: bool,
) {
    if arr.len() < 32 {
        // Small arrays: insertion sort
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }
    let pidx = match partition {
        "partition_left_left" => partition_left_left(arr, logger, pivot, optimized),
        "partition_left_right_pointers" => partition_left_right(arr, logger, pivot, optimized),
        other => {
            logger.mark(format!("Unknown partition '{}', falling back to left_left", other));
            partition_left_left(arr, logger, pivot, optimized)
        }
    };

    // Both partitions now use Lomuto and return the pivot final position
    // Split: smaller elements in [..pidx], pivot at pidx, larger elements in [pidx+1..]
    quick_sort_recursive(&mut arr[..pidx], logger, partition, pivot, optimized);
    quick_sort_recursive(&mut arr[pidx + 1..], logger, partition, pivot, optimized);
}

fn partition_left_left<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    pivot_strategy: &str,
    optimized: bool,
) -> usize {
    let len = arr.len();
    let pivot_idx = choose_pivot_index(arr, logger, pivot_strategy);
    // Move pivot to end
    logger.swap(arr, pivot_idx, len - 1);
    let pivot = arr[len - 1];

    // Optional small local optimization for last-element
    if optimized && pivot_strategy == "last_element" && len >= 4 {
        // Normalize last 3 elements relative order (similar to 3-swaps approach)
        if logger.cmp_lt(arr, len - 1, len - 2) { logger.swap(arr, len - 1, len - 2); }
        if logger.cmp_lt(arr, len - 1, len - 3) { logger.swap(arr, len - 1, len - 3); }
        if logger.cmp_lt(arr, len - 2, len - 1) { logger.swap(arr, len - 2, len - 1); }
    }

    let mut small = 0;
    for i in 0..len - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, len - 1);
    small
}

fn partition_left_right<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    pivot_strategy: &str,
    _optimized: bool,
) -> usize {
    // Use Lomuto partition for left-right pointers variant too
    // (for correct and predictable behavior)
    let len = arr.len();
    let pivot_idx = choose_pivot_index(arr, logger, pivot_strategy);
    // Move pivot to end
    logger.swap(arr, pivot_idx, len - 1);
    let pivot = arr[len - 1];

    let mut left = 0;
    for i in 0..len - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, left);
            left += 1;
        }
    }
    logger.swap(arr, left, len - 1);
    left
}

fn choose_pivot_index<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U, pivot: &str) -> usize {
    let len = arr.len();
    match pivot {
        "first_element" => 0,
        "last_element" => len - 1,
        "middle_element" => len / 2,
        "median_of_three" => {
            let a = 0;
            let b = len / 2;
            let c = len - 1;
            median_index(arr, logger, a, b, c)
        }
        "first_three" => {
            if len >= 3 { median_index(arr, logger, 0, 1, 2) } else { 0 }
        }
        "three_last" => {
            if len >= 3 { median_index(arr, logger, len - 3, len - 2, len - 1) } else { len - 1 }
        }
        "median_of_medians" => median_of_medians_index(arr, logger),
        other => {
            logger.mark(format!("Unknown pivot '{}' defaulting to middle", other));
            len / 2
        }
    }
}

fn median_index<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U, a: usize, b: usize, c: usize) -> usize {
    // Return the index whose value is the median among a,b,c
    let ab = logger.cmp_le(arr, a, b);
    let bc = logger.cmp_le(arr, b, c);
    let ac = logger.cmp_le(arr, a, c);
    match (ab, bc, ac) {
        (true, true, _) => { // a <= b <= c or a <= b and b <= c
            // median is b or a depending on ordering
            let ba = logger.cmp_le(arr, b, a);
            if ba { a } else { b }
        }
        (true, false, _) => { // a <= b and b > c
            // median is max(a,c)
            if logger.cmp_le(arr, a, c) { c } else { a }
        }
        (false, true, _) => { // a > b and b <= c
            // median is min(a,c)
            if logger.cmp_le(arr, a, c) { a } else { c }
        }
        (false, false, _) => { // a > b and b > c
            // median is b
            b
        }
    }
}

fn median_of_medians_index<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) -> usize {
    // Approximate: sample 5 evenly spaced indices and return median of their medians
    let len = arr.len();
    if len < 5 { return len / 2; }
    let samples = [0, len / 4, len / 2, (3 * len) / 4, len - 1];
    // Compute median among sampled indices using pairwise comparisons
    let m1 = median_index(arr, logger, samples[0], samples[1], samples[2]);
    let m2 = median_index(arr, logger, samples[2], samples[3], samples[4]);
    median_index(arr, logger, m1, samples[2], m2)
}
