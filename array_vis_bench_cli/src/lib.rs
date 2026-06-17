//! The interactive visualiser CLI — the picker + prompts + render call, lifted
//! verbatim out of the `array_vis_bench` binary so multiple link roots can share
//! it. It reads the registry purely by name / fn-pointer (`ALGORITHMS`,
//! `all_variants()`), so it neither knows nor cares whether the entries were
//! registered by combo_codegen, the spec compiler, or by hand. A binary is just
//! a link root that pulls in some registrations and calls [`run`].

use array_vis_bench_core::bench_registry::{list_inputs, Category, RunConfig};
use array_vis_bench_core::visualise::{find, visualise};
use array_vis_bench_traits::Complexity;
use sort_logger::VisualizerLogger;
use sort_registry_core::VariantDesc;
use sort_vis::{Encoding, Mp4Config, Pacing, COMMON_FRAMERATES, COMMON_RESOLUTIONS};
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

/// Run the interactive visualiser: pick a registered algorithm, choose an input,
/// a size, and render settings, then write `output.mp4`. The set of algorithms
/// shown is whatever the calling binary linked into the registry.
pub fn run() {
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
fn select_input(category: Category) -> String {
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

/// A faceted choice made so far: the axis `path` (the unique navigation key),
/// its `role` (for display), and the chosen `value`. Navigation keys on `path`
/// so a composite filler's sub-slots (`pivot/a`, `pivot/b`) are distinct levels
/// even when they share a role (`PivotSingle`).
#[derive(Clone)]
struct Pick {
    path: String,
    role: String,
    value: String,
}
type Chosen = Vec<Pick>;

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
/// `(role, value)` axes: at each step it presents a single axis — the next
/// unfixed role in **declaration order** (the type's parameter order) — never
/// mixing two axes in one list.
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

    // An axis is FIXED once its `path` has been chosen — keyed on path, not role,
    // so a composite's sub-slots (`pivot/a`, `pivot/b`) stay distinct even though
    // they share the role `PivotSingle`.
    let chosen_paths: HashSet<&str> = chosen.iter().map(|p| p.path.as_str()).collect();
    let resolved = |v: &VariantDesc| v.axes.iter().all(|a| chosen_paths.contains(a.path.as_str()));

    let mut opts: Vec<Opt> = Vec::new();
    let mut axis_role: Option<String> = None;
    let mut axis_path: Option<String> = None;
    let mut pending: Vec<(String, String)> = Vec::new();
    let mut label_tmpl: Option<&str> = None;

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
            .filter(|v| v.axes.iter().any(|a| !chosen_paths.contains(a.path.as_str())))
            .collect();
        if let Some((path, role, val_order, val_groups)) = next_facet(&axis_vars, &chosen_paths) {
            // Every still-unfixed axis other than the one in focus, in pre-order,
            // shown as holes so the partial type names all remaining parameters.
            for v in &axis_vars {
                for a in &v.axes {
                    if !chosen_paths.contains(a.path.as_str())
                        && a.path != path
                        && !pending.iter().any(|(p, _)| p == &a.path)
                    {
                        pending.push((a.path.clone(), a.role.clone()));
                    }
                }
            }
            // The catalog typed-label rendering only fits the FLAT top level (its
            // `{Role}` holes don't nest); once navigation descends into a
            // composite (a `<slot>/…` path), fall back to the generic breadcrumb.
            let nested = path.contains('/') || chosen.iter().any(|p| p.path.contains('/'));
            if !nested {
                if let Some(first) = cands.first().and_then(|v| v.label_template.as_deref()) {
                    if cands.iter().all(|v| v.label_template.as_deref() == Some(first)) {
                        label_tmpl = Some(first);
                    }
                }
            }
            for val in &val_order {
                let mut sub = chosen.clone();
                sub.push(Pick { path: path.clone(), role: role.clone(), value: val.clone() });
                opts.push(Opt::Branch {
                    label: val.clone(),
                    cands: val_groups[val].clone(),
                    depth,
                    chosen: sub,
                });
            }
            axis_role = Some(role);
            axis_path = Some(path);
        }
    }

    if opts.len() == 1 {
        return match opts.pop().unwrap() {
            Opt::Branch { cands, depth, chosen, .. } => pick(&cands, depth, &chosen),
            Opt::Leaf(name) => name.to_string(),
        };
    }

    let category = cands.first().map(|v| v.category.as_slice()).unwrap_or(&[]);
    print_breadcrumb(
        category,
        depth,
        chosen,
        axis_role.as_deref(),
        axis_path.as_deref(),
        &pending,
        label_tmpl,
    );
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

/// Compute the next faceted step among `axis_vars` (variants past the category
/// phase, with at least one unfixed axis): the axis `path` to present, its
/// display `role`, and the distinct values in pre-order with the candidates
/// carrying each. `None` once every axis is fixed. Pure — the unit-testable core
/// of the faceted navigation.
#[allow(clippy::type_complexity)]
fn next_facet<'a>(
    axis_vars: &[&'a VariantDesc],
    chosen_paths: &HashSet<&str>,
) -> Option<(String, String, Vec<String>, HashMap<String, Vec<&'a VariantDesc>>)> {
    let path = choose_axis_path(axis_vars, chosen_paths)?;
    let role = axis_vars
        .iter()
        .find_map(|v| v.axes.iter().find(|a| a.path == path).map(|a| a.role.clone()))
        .unwrap_or_default();
    let mut val_order: Vec<String> = Vec::new();
    let mut val_groups: HashMap<String, Vec<&VariantDesc>> = HashMap::new();
    for v in axis_vars {
        if let Some(ab) = v.axes.iter().find(|a| a.path == path) {
            if !val_groups.contains_key(&ab.value) {
                val_order.push(ab.value.clone());
            }
            val_groups.entry(ab.value.clone()).or_default().push(*v);
        }
    }
    Some((path, role, val_order, val_groups))
}

/// Pick the next axis to present: the first unfixed `path` in **pre-order** — the
/// order axes appear in each variant's list, which mirrors the type's parameter
/// tree (`QuickSort<Partition, Pivot, SmallSort>` → partition → pivot → (if a
/// composite) pivot's sub-slots → small-sort). Prefers a path present on *every*
/// candidate so the list never mixes axes that only some variants have — which is
/// also what defers a composite's sub-slots (`pivot/a`) until its parent
/// (`pivot`) is fixed to that composite.
fn choose_axis_path(axis_vars: &[&VariantDesc], chosen_paths: &HashSet<&str>) -> Option<String> {
    let n = axis_vars.len();
    let mut order: Vec<&str> = Vec::new();
    let mut present: HashMap<&str, usize> = HashMap::new();
    for v in axis_vars {
        for a in &v.axes {
            let path = a.path.as_str();
            if !chosen_paths.contains(path) {
                if !present.contains_key(path) {
                    order.push(path);
                }
                *present.entry(path).or_default() += 1;
            }
        }
    }
    order
        .iter()
        .find(|p| present[**p] == n)
        .or_else(|| order.first())
        .map(|p| p.to_string())
}

/// Print the partial type being built.
///
/// When the candidates share a catalog label template (the faceted phase of a
/// spec-emitted family), render it in the catalog's **own label syntax** —
/// head, per-slot labels, brackets — via [`print_typed_label`]. Otherwise
/// (category navigation, or combo / legacy entries with no template) fall back
/// to the generic display: each consumed category segment opens a bracket,
/// filled parameters show `role: value,`, the hole being chosen shows
/// `role: _,`, and every parameter still owed shows `role: -,`.
///
/// In the faceted phase the parameter list is complete (filled or hole), so the
/// opened brackets are closed and the partial type is balanced. In the category
/// phase the menu below *is* the next nesting level, so the innermost bracket
/// stays open for it.
fn print_breadcrumb(
    category: &[String],
    depth: usize,
    chosen: &Chosen,
    axis_role: Option<&str>,
    axis_path: Option<&str>,
    pending: &[(String, String)],
    label_template: Option<&str>,
) {
    println!();
    if let Some(tmpl) = label_template {
        print_typed_label(tmpl, chosen, axis_role);
        return;
    }
    let depth = depth.min(category.len());
    let mut indent = 0;
    for seg in &category[..depth] {
        println!("{}{}<", "  ".repeat(indent), seg);
        indent += 1;
    }
    let pad = "  ".repeat(indent);
    // A nested axis (path `pivot/a`) displays its slot tail (`a`) so two sub-slots
    // sharing a role (`PivotSingle`) read distinctly; a top-level axis shows its role.
    let key = |path: &str, role: &str| -> String {
        match path.rsplit_once('/') {
            Some((_, tail)) => tail.to_string(),
            None => role.to_string(),
        }
    };
    for p in chosen {
        println!("{pad}{}: {},", key(&p.path, &p.role), p.value);
    }
    if let Some(role) = axis_role {
        println!("{pad}{}: _,", key(axis_path.unwrap_or(role), role));
    }
    for (path, role) in pending {
        println!("{pad}{}: -,", key(path, role));
    }
    // Faceted phase: the parameter list is complete, so close every bracket we
    // opened. Category phase (no axes chosen, no hole) leaves them open — the
    // menu below fills the innermost one.
    if axis_role.is_some() || !chosen.is_empty() {
        for d in (0..indent).rev() {
            println!("{}>", "  ".repeat(d));
        }
    }
}

/// Render a partial type from a role-tagged label template — e.g.
/// `spec::quick sort<part: {Partition}, pivot: {Pivot}, small: {SmallSort}>` —
/// one parameter per line. Each `{Role}` hole becomes its chosen value, `_` for
/// the slot being chosen now, or `-` for a still-owed slot; a fragment with no
/// hole prints verbatim. The brackets are always balanced, so the partial type
/// stays well-formed.
fn print_typed_label(template: &str, chosen: &Chosen, current: Option<&str>) {
    let chosen_val =
        |role: &str| chosen.iter().find(|p| p.role == role).map(|p| p.value.as_str());

    let Some(open) = template.find('<') else {
        // No generic parameters — the label itself is the whole partial type.
        println!("{template}");
        return;
    };
    let close = template.rfind('>').unwrap_or(template.len());
    let head = &template[..open];
    let body = &template[open + 1..close];
    let tail = &template[close..];

    println!("{head}<");
    for frag in split_top_level(body) {
        let frag = frag.trim();
        if frag.is_empty() {
            continue;
        }
        match (frag.find('{'), frag.find('}')) {
            (Some(lb), Some(rb)) if lb < rb => {
                let role = &frag[lb + 1..rb];
                let prefix = &frag[..lb];
                let suffix = &frag[rb + 1..];
                // Filled slot → its value; the slot chosen now → `_`; a slot
                // still owed → `-`.
                let fill = if let Some(val) = chosen_val(role) {
                    val
                } else if current == Some(role) {
                    "_"
                } else {
                    "-"
                };
                println!("  {prefix}{fill}{suffix},");
            }
            // A fragment with no `{Role}` hole (e.g. a const baked into the
            // label): show it literally.
            _ => println!("  {frag},"),
        }
    }
    println!("{tail}");
}

/// Split on top-level commas, ignoring those nested inside `<…>` / `(…)` /
/// `[…]`, so a value like `combined<first,mid>` stays a single fragment.
fn split_top_level(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '>' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sort_registry_core::{AxisBinding, VariantDesc};

    fn var(name: &str, axes: &[(&str, &str, &str)]) -> VariantDesc {
        VariantDesc {
            name: name.into(),
            category: vec!["sorts".into(), "quick sorts".into()],
            axes: axes
                .iter()
                .map(|(r, v, p)| AxisBinding {
                    role: r.to_string(),
                    value: v.to_string(),
                    path: p.to_string(),
                })
                .collect(),
            label_template: None,
        }
    }

    /// The structural nav: a composite filler (`combined<a,b>`) is its own level,
    /// then its sub-slots `a`/`b` drill in — keyed on `path`, not `role` (both
    /// sub-slots share role `PivotSingle`).
    #[test]
    fn combined_pivot_is_its_own_level_then_a_then_b() {
        let v1 = var("c1", &[
            ("Partition", "dual", "partition"), ("Pivot", "combined", "pivot"),
            ("PivotSingle", "first", "pivot/a"), ("PivotSingle", "first", "pivot/b"),
            ("SmallSort", "s", "small_sort"),
        ]);
        let v2 = var("c2", &[
            ("Partition", "dual", "partition"), ("Pivot", "combined", "pivot"),
            ("PivotSingle", "middle", "pivot/a"), ("PivotSingle", "first", "pivot/b"),
            ("SmallSort", "s", "small_sort"),
        ]);
        let vn = var("n", &[
            ("Partition", "dual", "partition"), ("Pivot", "ninther", "pivot"),
            ("SmallSort", "s", "small_sort"),
        ]);

        // After partition=dual, the pivot axis groups to {combined, ninther} —
        // all combined pairs collapse under the one head `combined`.
        let cands = [&v1, &v2, &vn];
        let chosen: HashSet<&str> = ["partition"].into_iter().collect();
        let (path, role, vals, groups) = next_facet(&cands, &chosen).unwrap();
        assert_eq!((path.as_str(), role.as_str()), ("pivot", "Pivot"));
        assert_eq!(vals, vec!["combined".to_string(), "ninther".to_string()]);
        assert_eq!(groups["combined"].len(), 2);

        // Descend into combined → the `a` sub-slot is next, over {first, middle}.
        let combined = [&v1, &v2];
        let chosen2: HashSet<&str> = ["partition", "pivot"].into_iter().collect();
        let (path_a, _r, vals_a, _g) = next_facet(&combined, &chosen2).unwrap();
        assert_eq!(path_a, "pivot/a");
        assert_eq!(vals_a, vec!["first".to_string(), "middle".to_string()]);
    }

    /// A single (leaf) pivot has no sub-slots, so navigation goes straight from
    /// `pivot` to `small_sort` — `pivot/a` never appears.
    #[test]
    fn leaf_pivot_skips_to_small_sort() {
        let s = var("s", &[
            ("Partition", "left-left", "partition"), ("Pivot", "first", "pivot"),
            ("SmallSort", "size: 1", "small_sort"),
        ]);
        let cands = [&s];
        let chosen: HashSet<&str> = ["partition", "pivot"].into_iter().collect();
        let (path, _r, _v, _g) = next_facet(&cands, &chosen).unwrap();
        assert_eq!(path, "small_sort");
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
