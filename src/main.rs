use array_vis_bench_full::bench_registry::{list_inputs, RunConfig};
use array_vis_bench_full::traits::complexity::Complexity;
use array_vis_bench_full::traits::log_traits::VisualizerLogger;
use array_vis_bench_full::visualise::{find, visualise};
use sort_registry_core::VariantDesc;
use sort_vis::{Encoding, Mp4Config, Pacing, COMMON_FRAMERATES, COMMON_RESOLUTIONS};
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

fn main() {
    println!("Array Visualization Benchmark");
    println!("==============================");

    let sort_name = select_sort();
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

type Chosen = Vec<(String, String)>;

fn select_sort() -> String {
    let variants = sort_registry_core::all_variants();
    let cands: Vec<&VariantDesc> = variants.iter().collect();
    pick(&cands, 0, &Vec::new())
}

/// One step of the picker.
///
/// `depth` is how many `category` segments have been consumed. While any
/// candidate still has a category segment at `depth`, this is structural
/// navigation (one branch per distinct segment). Once category is
/// exhausted, it switches to **faceted** navigation over the variants'
/// `(role, value)` axes: at each step it presents the single axis with the
/// most distinct values (max-first), never mixing two axes in one list.
fn pick(cands: &[&VariantDesc], depth: usize, chosen: &Chosen) -> String {
    enum Opt<'a> {
        Branch { label: String, cands: Vec<&'a VariantDesc>, depth: usize, chosen: Chosen },
        Leaf(&'a str),
    }

    let mut cat_order: Vec<String> = Vec::new();
    let mut cat_groups: HashMap<String, Vec<&VariantDesc>> = HashMap::new();
    let mut faceted: Vec<&VariantDesc> = Vec::new();
    for v in cands {
        if v.category.len() > depth {
            let key = v.category[depth].clone();
            if !cat_groups.contains_key(&key) {
                cat_order.push(key.clone());
            }
            cat_groups.entry(key).or_default().push(*v);
        } else {
            faceted.push(*v);
        }
    }

    let chosen_roles: HashSet<&str> = chosen.iter().map(|(r, _)| r.as_str()).collect();
    let resolved = |v: &VariantDesc| v.axes.iter().all(|a| chosen_roles.contains(a.role.as_str()));

    let mut opts: Vec<Opt> = Vec::new();
    let mut axis_role: Option<String> = None;

    if !cat_groups.is_empty() {
        for key in &cat_order {
            opts.push(Opt::Branch {
                label: key.clone(),
                cands: cat_groups[key].clone(),
                depth: depth + 1,
                chosen: chosen.clone(),
            });
        }
        for &v in &faceted {
            if resolved(v) {
                opts.push(Opt::Leaf(&v.name));
            }
        }
    } else {
        for &v in &faceted {
            if resolved(v) {
                opts.push(Opt::Leaf(&v.name));
            }
        }
        let axis_vars: Vec<&VariantDesc> = faceted
            .iter()
            .copied()
            .filter(|v| v.axes.iter().any(|a| !chosen_roles.contains(a.role.as_str())))
            .collect();
        if !axis_vars.is_empty() {
            let role = choose_axis(&axis_vars, &chosen_roles);
            let mut val_order: Vec<String> = Vec::new();
            let mut val_groups: HashMap<String, Vec<&VariantDesc>> = HashMap::new();
            for v in &axis_vars {
                if let Some(ab) = v.axes.iter().find(|a| a.role == role) {
                    if !val_groups.contains_key(&ab.value) {
                        val_order.push(ab.value.clone());
                    }
                    val_groups.entry(ab.value.clone()).or_default().push(*v);
                }
            }
            for val in &val_order {
                let mut sub_chosen = chosen.clone();
                sub_chosen.push((role.clone(), val.clone()));
                opts.push(Opt::Branch {
                    label: val.clone(),
                    cands: val_groups[val].clone(),
                    depth,
                    chosen: sub_chosen,
                });
            }
            axis_role = Some(role);
        }
    }

    if opts.len() == 1 {
        return match opts.pop().unwrap() {
            Opt::Branch { cands, depth, chosen, .. } => pick(&cands, depth, &chosen),
            Opt::Leaf(name) => name.to_string(),
        };
    }

    let category = cands.first().map(|v| v.category.as_slice()).unwrap_or(&[]);
    print_breadcrumb(category, depth, chosen, axis_role.as_deref());
    for (i, opt) in opts.iter().enumerate() {
        match opt {
            Opt::Branch { label, cands, .. } => {
                let n = cands.len();
                let range = complexity_range(cands);
                println!(
                    "  {}: {} ({} variant{}){}",
                    i + 1, label, n,
                    if n == 1 { "" } else { "s" },
                    range,
                );
            }
            Opt::Leaf(name) => {
                let suffix = find(name)
                    .filter(|e| !e.average.is_unknown())
                    .map(|e| format!(" ({})", e.average.as_str()))
                    .unwrap_or_default();
                println!("  {}: {}{}", i + 1, name, suffix);
            }
        }
    }
    let sel = get_user_selection("Select", 1, opts.len()) - 1;
    match opts.into_iter().nth(sel).unwrap() {
        Opt::Branch { cands, depth, chosen, .. } => pick(&cands, depth, &chosen),
        Opt::Leaf(name) => name.to_string(),
    }
}

/// Pick the next axis to present: the unfixed role with the most distinct
/// values (max-first). Prefers roles present on *every* candidate so the
/// list never mixes axes that only some variants have.
fn choose_axis(axis_vars: &[&VariantDesc], chosen_roles: &HashSet<&str>) -> String {
    let n = axis_vars.len();
    let mut distinct: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut present: HashMap<&str, usize> = HashMap::new();
    for v in axis_vars {
        for a in &v.axes {
            if !chosen_roles.contains(a.role.as_str()) {
                distinct.entry(&a.role).or_default().insert(&a.value);
                *present.entry(&a.role).or_default() += 1;
            }
        }
    }
    let universal: Vec<&str> =
        present.iter().filter(|(_, c)| **c == n).map(|(r, _)| *r).collect();
    let pool: Vec<&str> = if universal.is_empty() {
        present.keys().copied().collect()
    } else {
        universal
    };
    pool.into_iter()
        .max_by(|a, b| distinct[a].len().cmp(&distinct[b].len()).then_with(|| b.cmp(a)))
        .unwrap()
        .to_string()
}

/// Print the partial type being built, one element per line, with the
/// current hole marked `_`.
fn print_breadcrumb(category: &[String], depth: usize, chosen: &Chosen, axis_role: Option<&str>) {
    println!();
    let mut indent = 0;
    for seg in &category[..depth.min(category.len())] {
        println!("{}{}<", "  ".repeat(indent), seg);
        indent += 1;
    }
    for (role, val) in chosen {
        println!("{}{}: {},", "  ".repeat(indent), role, val);
    }
    if let Some(role) = axis_role {
        println!("{}{}: _", "  ".repeat(indent), role);
    }
}

/// " (O(min) - O(max))" tag describing the spread of average-case
/// complexities across the variants. Returns "" when none resolve.
fn complexity_range(cands: &[&VariantDesc]) -> String {
    let mut min: Option<Complexity> = None;
    let mut max: Option<Complexity> = None;
    for v in cands {
        if let Some(entry) = find(&v.name) {
            let a = entry.average;
            if a.is_unknown() {
                continue;
            }
            min = Some(min.map_or(a, |m| if cmp_complexity(a, m).is_lt() { a } else { m }));
            max = Some(max.map_or(a, |m| Complexity::sum(a, m)));
        }
    }
    match (min, max) {
        (Some(lo), Some(hi)) if lo == hi => format!(" ({})", lo.as_str()),
        (Some(lo), Some(hi)) => format!(" ({} - {})", lo.as_str(), hi.as_str()),
        _ => String::new(),
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
