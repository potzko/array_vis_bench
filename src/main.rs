use array_vis_bench_full::bench_registry::{list_inputs, RunConfig};
use array_vis_bench_full::traits::complexity::Complexity;
use array_vis_bench_full::traits::get_sort_tree;
use array_vis_bench_full::traits::log_traits::VisualizerLogger;
use array_vis_bench_full::visualise::{find, visualise};
use sort_vis::{Encoding, Mp4Config, Pacing, COMMON_FRAMERATES, COMMON_RESOLUTIONS};
use std::io::{self, Write};

fn main() {
    println!("Array Visualization Benchmark");
    println!("==============================");

    let tree = get_sort_tree();
    let sort_name = select_sort(&tree);
    println!("Selected: {sort_name}");

    let entry = find(&sort_name).unwrap_or_else(|| {
        panic!("algorithm '{}' not registered in ALGORITHMS", sort_name)
    });
    let input_name = select_input(entry.category);
    println!("Input: {input_name}");
    let size = read_size_with_cap(500, entry.max_input_size);

    println!("\nVideo resolution:");
    for (i, (w, h, label)) in COMMON_RESOLUTIONS.iter().enumerate() {
        println!("  {}: {}x{} ({})", i + 1, w, h, label);
    }
    // 1080p (index 2) is the right default for almost every preview —
    // 720p loses too much detail at high element counts, the higher
    // tiers are slow without a meaningful gain on a laptop screen.
    let res_idx = get_user_selection_with_default(
        "Select resolution", 1, COMMON_RESOLUTIONS.len(), 2,
    ) - 1;
    let (output_width, output_height, _) = COMMON_RESOLUTIONS[res_idx];

    println!("\nFrame rate:");
    for (i, fr) in COMMON_FRAMERATES.iter().enumerate() {
        println!("  {}: {} fps", i + 1, fr);
    }
    // 60 fps (index 2): smooth playback without doubling the file
    // size you'd get at 120.
    let fr_idx = get_user_selection_with_default(
        "Select frame rate", 1, COMMON_FRAMERATES.len(), 2,
    ) - 1;
    let framerate = COMMON_FRAMERATES[fr_idx];

    print!("\nHow long should the visualization be (seconds): ");
    io::stdout().flush().unwrap();
    let mut duration_input = String::new();
    io::stdin().read_line(&mut duration_input).unwrap();
    let duration_secs: f64 = duration_input.trim().parse().unwrap_or_else(|_| {
        println!("Invalid input, defaulting to 60s.");
        60.0
    });

    let mp4_config = Mp4Config {
        output_width,
        output_height,
        framerate,
        pacing: Pacing::DurationSeconds(duration_secs),
        encoding: Encoding::Fast,
        output_path: "output.mp4".into(),
    };

    let run_config = RunConfig { size, seed: 0 };

    let mut logger = VisualizerLogger {
        log: Vec::new(),
        type_ghost: std::marker::PhantomData,
    };

    println!("\nGenerating visualization...");
    visualise(&sort_name, &input_name, &run_config, &mut logger, mp4_config);

    println!("  - Operations logged: {}", logger.log.len());
    println!("  - Output saved as: output.mp4");
    println!("\nVisualization complete!");
}

/// Show the registered inputs for `category` and let the user pick one.
fn select_input(category: array_vis_bench_full::bench_registry::Category) -> String {
    let names = list_inputs(category);
    assert!(
        !names.is_empty(),
        "no inputs registered for category {:?}",
        category
    );
    println!("\nInput shapes for {}:", category.as_str());
    for (i, name) in names.iter().enumerate() {
        println!("  {}: {}", i + 1, name);
    }
    let idx = get_user_selection("Select input", 1, names.len()) - 1;
    names[idx].to_string()
}

/// Prompt for array size. If `cap` is `Some(n)`, the prompt advertises
/// the cap and clamps both the default and any user-supplied size to
/// `n` — small-sorts and other contract-bounded algorithms register a
/// cap so the visualiser doesn't ask for sizes the algorithm refuses
/// to handle.
fn read_size_with_cap(default: usize, cap: Option<usize>) -> usize {
    let effective_default = cap.map(|c| default.min(c)).unwrap_or(default);
    let prompt_suffix = match cap {
        Some(c) => format!(" [default {effective_default}, max {c}]"),
        None => format!(" [default {effective_default}]"),
    };
    print!("\nArray size{prompt_suffix}: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let raw = input.trim().parse().unwrap_or(effective_default);
    cap.map(|c| raw.min(c)).unwrap_or(raw)
}

fn get_user_selection(prompt: &str, min: usize, max: usize) -> usize {
    get_user_selection_inner(prompt, min, max, None)
}

/// Like `get_user_selection`, but an empty line accepts `default`
/// instead of looping. The prompt advertises the default in brackets.
fn get_user_selection_with_default(
    prompt: &str,
    min: usize,
    max: usize,
    default: usize,
) -> usize {
    get_user_selection_inner(prompt, min, max, Some(default))
}

fn get_user_selection_inner(
    prompt: &str,
    min: usize,
    max: usize,
    default: Option<usize>,
) -> usize {
    loop {
        match default {
            Some(d) => print!("\n{prompt} ({min} to {max}) [default {d}]: "),
            None => print!("\n{prompt} ({min} to {max}): "),
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input; try again.");
            continue;
        }
        let trimmed = input.trim();
        if trimmed.is_empty() {
            if let Some(d) = default {
                return d;
            }
            println!("Error: please enter a number.");
            continue;
        }
        match trimmed.parse::<usize>() {
            Ok(n) if n >= min && n <= max => return n,
            Ok(n) => println!("Error: {n} is out of range [{min}, {max}]."),
            Err(_) => println!("Error: '{trimmed}' is not a number."),
        }
    }
}

fn select_sort(tree: &sort_registry_core::SortTree) -> String {
    select_sort_inner(tree)
}

fn select_sort_inner(tree: &sort_registry_core::SortTree) -> String {
    enum Opt<'a> {
        Branch(&'a str, &'a sort_registry_core::SortTree),
        Leaf(&'a str, &'a str),
    }
    let mut opts: Vec<Opt> = Vec::new();
    for (label, sub) in &tree.children {
        opts.push(Opt::Branch(label, sub));
    }
    for (display, name) in &tree.leaves {
        opts.push(Opt::Leaf(display, name));
    }
    if opts.len() == 1 {
        return match &opts[0] {
            Opt::Branch(_, sub) => select_sort_inner(sub),
            Opt::Leaf(_, name) => name.to_string(),
        };
    }
    println!();
    for (i, opt) in opts.iter().enumerate() {
        match opt {
            Opt::Branch(label, sub) => {
                let n = sub.count_leaves();
                let range = average_complexity_range(sub);
                println!(
                    "  {}: {} ({} variant{}){}",
                    i + 1, label, n,
                    if n == 1 { "" } else { "s" },
                    range,
                );
            }
            Opt::Leaf(display, name) => {
                let suffix = find(name)
                    .map(|e| format!(" ({})", e.average.as_str()))
                    .unwrap_or_default();
                println!("  {}: {}{}", i + 1, display, suffix);
            }
        }
    }
    let sel = get_user_selection("Select", 1, opts.len()) - 1;
    match &opts[sel] {
        Opt::Branch(_, sub) => select_sort_inner(sub),
        Opt::Leaf(_, name) => name.to_string(),
    }
}

/// Walk every leaf in `tree`, look up each name in `ALGORITHMS`, and
/// return a printable " (O(min) - O(max))" tag describing the spread of
/// average-case complexities. Returns "" when no leaves resolve to an
/// entry (the picker degrades to its previous behaviour for unknown
/// trees rather than printing "(O(?) - O(?))").
fn average_complexity_range(tree: &sort_registry_core::SortTree) -> String {
    let mut min: Option<Complexity> = None;
    let mut max: Option<Complexity> = None;
    walk_leaves(tree, &mut |name| {
        if let Some(entry) = find(name) {
            let a = entry.average;
            min = Some(min.map_or(a, |m| if cmp_complexity(a, m).is_lt() { a } else { m }));
            max = Some(max.map_or(a, |m| Complexity::sum(a, m)));
        }
    });
    match (min, max) {
        (Some(lo), Some(hi)) if lo == hi => format!(" ({})", lo.as_str()),
        (Some(lo), Some(hi)) => format!(" ({} - {})", lo.as_str(), hi.as_str()),
        _ => String::new(),
    }
}

fn walk_leaves(tree: &sort_registry_core::SortTree, f: &mut dyn FnMut(&str)) {
    for (_, name) in &tree.leaves {
        f(name);
    }
    for (_, sub) in &tree.children {
        walk_leaves(sub, f);
    }
}

/// Order on big-O classes that matches `Complexity::sum` (which returns
/// the dominant of two). Used to find the minimum complexity in a
/// branch — `sum` only gives us the max.
fn cmp_complexity(a: Complexity, b: Complexity) -> std::cmp::Ordering {
    if a == b {
        std::cmp::Ordering::Equal
    } else if Complexity::sum(a, b) == a {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Less
    }
}
