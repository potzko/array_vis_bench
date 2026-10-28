//! The interactive visualiser CLI — the picker + prompts + render call, lifted
//! verbatim out of the `array_vis_bench` binary so multiple link roots can share
//! it. It reads the registry purely by name / fn-pointer (`ALGORITHMS` and the
//! picker view `navigation::variants()` built from it), so it neither knows nor
//! cares whether the entries were registered by the spec compiler or by hand. A
//! binary is just a link root that pulls in some registrations and calls [`run`].

use array_vis_bench_core::bench_registry::{list_inputs, Category, RunConfig};
use array_vis_bench_core::navigation::{step, variants, Choice, Chosen, VariantDesc};
use array_vis_bench_core::visualise::{find, visualise};
use array_vis_bench_traits::Complexity;
use sort_logger::VisualizerLogger;
use sort_vis::{Encoding, Mp4Config, Pacing, COMMON_FRAMERATES, COMMON_RESOLUTIONS};
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
fn select_sort() -> String {
    let all = variants();
    let cands: Vec<&VariantDesc> = all.iter().collect();
    pick(&cands, 0, &Vec::new())
}

/// One step of the picker: the choices `navigation::step` computes, a lone
/// choice descended without asking, otherwise the breadcrumb, the list and
/// the prompt.
fn pick(cands: &[&VariantDesc], depth: usize, chosen: &Chosen) -> String {
    let mut now = step(cands, depth, chosen);
    if now.choices.len() == 1 {
        return descend(now.choices.pop().unwrap());
    }
    let category = cands.first().map(|v| v.category.as_slice()).unwrap_or(&[]);
    let (axis_path, axis_role) = match &now.axis {
        Some((path, role)) => (Some(path.as_str()), Some(role.as_str())),
        None => (None, None),
    };
    print_breadcrumb(category, depth, chosen, axis_role, axis_path, &now.pending, now.label_template);
    for (i, choice) in now.choices.iter().enumerate() {
        match choice {
            Choice::Branch { label, cands, .. } => {
                let n = cands.len();
                let range = complexity_range(cands);
                println!(
                    "  {}: {} ({} variant{}){}",
                    i + 1, label, n,
                    if n == 1 { "" } else { "s" },
                    range,
                );
            }
            Choice::Leaf(v) => {
                let suffix = find(&v.name)
                    .filter(|e| !e.average.is_unknown())
                    .map(|e| format!(" ({})", e.average.as_str()))
                    .unwrap_or_default();
                println!("  {}: {}{}", i + 1, v.name, suffix);
            }
        }
    }
    let sel = get_user_selection("Select", 1, now.choices.len()) - 1;
    descend(now.choices.into_iter().nth(sel).unwrap())
}

fn descend(choice: Choice) -> String {
    match choice {
        Choice::Branch { cands, depth, chosen, .. } => pick(&cands, depth, &chosen),
        Choice::Leaf(v) => v.name.clone(),
    }
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
    print!("{}", render_typed_label(template, chosen, current));
}

/// Build the partial-type string [`print_typed_label`] emits — factored out so
/// the slot rendering is unit-testable without capturing stdout.
fn render_typed_label(template: &str, chosen: &Chosen, current: Option<&str>) -> String {
    let chosen_val =
        |role: &str| chosen.iter().find(|p| p.role == role).map(|p| p.value.as_str());

    let Some(open) = template.find('<') else {
        // No generic parameters — the label itself is the whole partial type.
        return format!("{template}\n");
    };
    let close = template.rfind('>').unwrap_or(template.len());
    let head = &template[..open];
    let body = &template[open + 1..close];
    let tail = &template[close..];

    let mut out = format!("{head}<\n");
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
                // A bare `{role}` hole (no label text around it, as in
                // `quick select<{partition}, {pivot}>`) would otherwise print
                // just the value; prefix the role name so each slot reads
                // `partition = block`. Holes with their own label text (e.g.
                // `part: {pivot}`) already name the slot, so leave them alone.
                if prefix.is_empty() && suffix.is_empty() {
                    out.push_str(&format!("  {role} = {fill},\n"));
                } else {
                    out.push_str(&format!("  {prefix}{fill}{suffix},\n"));
                }
            }
            // A fragment with no `{Role}` hole (e.g. a const baked into the
            // label): show it literally.
            _ => out.push_str(&format!("  {frag},\n")),
        }
    }
    out.push_str(&format!("{tail}\n"));
    out
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
    use array_vis_bench_core::navigation::Pick;

    fn pick(role: &str, value: &str) -> Pick {
        Pick { path: role.into(), role: role.into(), value: value.into() }
    }

    /// Bare `{role}` holes (quick select's `<{partition}, {pivot}>`) render with
    /// the role name, so each slot reads `partition = block` rather than a bare
    /// value.
    #[test]
    fn bare_holes_render_the_role_name() {
        let tmpl = "quick select: iterative<{partition}, {pivot}>";
        let chosen = vec![pick("partition", "block")];
        let out = render_typed_label(tmpl, &chosen, Some("pivot"));
        assert_eq!(
            out,
            "quick select: iterative<\n  partition = block,\n  pivot = _,\n>\n",
        );
    }

    /// Holes that already carry their own label text (quick sort's `part: {…}`)
    /// keep the catalog's `role: value` form — the role-name prefix is only for
    /// bare holes.
    #[test]
    fn labeled_holes_keep_their_own_text() {
        let tmpl = "quick sort<part: {Partition}, pivot: {Pivot}>";
        let chosen = vec![pick("Partition", "block")];
        let out = render_typed_label(tmpl, &chosen, Some("Pivot"));
        assert_eq!(out, "quick sort<\n  part: block,\n  pivot: _,\n>\n");
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
