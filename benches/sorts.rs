use std::collections::HashMap;
use std::fs::File;
use std::time::{Duration, Instant};

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;

use array_vis_bench::bench_registry;
use array_vis_bench::utils::array_gen::get_rand_arr;
use sort_registry_core::{get_sort_tree, SortTree};

/// Base per-(sort × N) time budget. Sorts whose probe average exceeds the
/// threshold are dropped from future N levels. The *current* threshold
/// grows as the active set shrinks (see [`current_threshold`]) so the
/// fastest sorts can be compared at large N without being bounded by the
/// budget that was reasonable when 1000+ sorts were still in play.
const BASE_SLOW_THRESHOLD: Duration = Duration::from_millis(100);

/// Hard ceiling for the adaptive threshold.
const MAX_SLOW_THRESHOLD: Duration = Duration::from_secs(5);

/// Number of timed runs used to decide whether a sort is too slow for this N.
/// Also acts as warmup before the sample loop.
const PROBE_RUNS: usize = 3;

/// Timed runs collected per (sort × N) pair that passes the threshold.
const SAMPLES: usize = 10;

/// Binary-search steps to refine the cliff for a dropped sort. Three steps
/// in a 2× window narrows the cliff to ~1/8 of the gap.
const REFINE_STEPS: usize = 3;

const MAX_N: usize = 5_000_000_000;

const ARCHIVE_PATH: &str = "target/bench_archive.json";

#[derive(Serialize)]
struct Record {
    name: &'static str,
    n: usize,
    mean_ns: f64,
    stderr_ns: f64,
}

#[derive(Serialize)]
struct ThresholdPoint {
    n: usize,
    threshold_ns: u64,
}

#[derive(Serialize)]
struct TreeNode {
    label: String,
    is_leaf: bool,
    /// Only set on leaves — the registered sort name (matches `Record.name`).
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    children: Vec<TreeNode>,
}

fn build_tree(node: &SortTree, label: String) -> TreeNode {
    let mut children: Vec<TreeNode> = node
        .children
        .iter()
        .map(|(child_label, child)| build_tree(child, child_label.clone()))
        .collect();
    for (leaf_label, sort_name) in &node.leaves {
        children.push(TreeNode {
            label: leaf_label.clone(),
            is_leaf: true,
            name: Some(sort_name.clone()),
            children: Vec::new(),
        });
    }
    TreeNode {
        label,
        is_leaf: false,
        name: None,
        children,
    }
}

fn current_threshold(initial: usize, active: usize) -> Duration {
    // sqrt dampens the growth: dropping half the sorts only ~1.4× the budget
    // (vs. 2× with a linear multiplier). Keeps small-set N levels from
    // ballooning into multi-second-per-sort territory.
    let multiplier = (initial as f64 / active.max(1) as f64).max(1.0).sqrt();
    BASE_SLOW_THRESHOLD.mul_f64(multiplier).min(MAX_SLOW_THRESHOLD)
}

/// Probe + sample one sort against a pre-allocated buffer.
/// Returns `None` if the probe average exceeded the threshold.
fn measure(
    entry: &bench_registry::SortBenchEntry,
    buf: &mut [usize],
    threshold: Duration,
    rng: &mut ThreadRng,
) -> Option<(f64, f64)> {
    let mut probe_total = Duration::ZERO;
    for _ in 0..PROBE_RUNS {
        buf.shuffle(rng);
        let t = Instant::now();
        (entry.run)(buf);
        probe_total += t.elapsed();
    }
    if probe_total / PROBE_RUNS as u32 > threshold {
        return None;
    }

    let mut samples = [0u64; SAMPLES];
    for s in &mut samples {
        buf.shuffle(rng);
        let t = Instant::now();
        (entry.run)(buf);
        *s = t.elapsed().as_nanos() as u64;
    }
    let mean = samples.iter().copied().sum::<u64>() as f64 / SAMPLES as f64;
    let var = samples
        .iter()
        .map(|&x| {
            let d = x as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / SAMPLES as f64;
    let stderr = (var / SAMPLES as f64).sqrt();
    Some((mean, stderr))
}

fn main() {
    let mut active: Vec<&'static bench_registry::SortBenchEntry> =
        bench_registry::BENCH_SORTS.iter().collect();
    let initial_active = active.len();

    let mut rng = thread_rng();
    let mut n = 10usize;
    let mut records: Vec<Record> = Vec::new();
    let mut thresholds: Vec<ThresholdPoint> = Vec::new();
    let mut last_pass_n: HashMap<&'static str, usize> = HashMap::new();

    let tree = build_tree(&get_sort_tree(), "all".to_string());

    while n <= MAX_N && !active.is_empty() {
        let threshold = current_threshold(initial_active, active.len());
        thresholds.push(ThresholdPoint {
            n,
            threshold_ns: threshold.as_nanos() as u64,
        });
        eprintln!(
            "── n={n}  active={}  threshold={:?}",
            active.len(),
            threshold
        );

        let mut buf = get_rand_arr(n);
        let mut too_slow: Vec<&'static bench_registry::SortBenchEntry> = Vec::new();

        for entry in &active {
            match measure(entry, &mut buf, threshold, &mut rng) {
                Some((mean, stderr)) => {
                    println!(
                        "n={n:<10} {:>12.0} ns  ± {:<10.0}  {}",
                        mean, stderr, entry.name
                    );
                    records.push(Record {
                        name: entry.name,
                        n,
                        mean_ns: mean,
                        stderr_ns: stderr,
                    });
                    last_pass_n.insert(entry.name, n);
                }
                None => too_slow.push(entry),
            }
        }
        drop(buf);

        // Binary-search refinement: for each sort that just got dropped,
        // probe inside [last_pass_n, n] to capture data around the cliff.
        for entry in &too_slow {
            let Some(&n_low) = last_pass_n.get(entry.name) else {
                continue; // never passed — nothing to refine
            };
            let mut lo = n_low;
            let mut hi = n;
            for _ in 0..REFINE_STEPS {
                if hi - lo <= 1 {
                    break;
                }
                let mid = (lo + hi) / 2;
                let mut buf_mid = get_rand_arr(mid);
                match measure(entry, &mut buf_mid, threshold, &mut rng) {
                    Some((mean, stderr)) => {
                        println!(
                            "n={mid:<10} {:>12.0} ns  ± {:<10.0}  {}  (refine)",
                            mean, stderr, entry.name
                        );
                        records.push(Record {
                            name: entry.name,
                            n: mid,
                            mean_ns: mean,
                            stderr_ns: stderr,
                        });
                        lo = mid;
                    }
                    None => {
                        hi = mid;
                    }
                }
            }
        }

        active.retain(|e| !too_slow.iter().any(|t| t.name == e.name));

        if !too_slow.is_empty() {
            eprintln!(
                "n={n}: dropped {} sort(s): {}",
                too_slow.len(),
                too_slow
                    .iter()
                    .map(|e| e.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        write_archive(&records, &thresholds, &tree);
        n *= 2;
    }

    println!("\nWrote {} records to {}", records.len(), ARCHIVE_PATH);
}

fn write_archive(records: &[Record], thresholds: &[ThresholdPoint], tree: &TreeNode) {
    std::fs::create_dir_all("target").ok();
    let file = File::create(ARCHIVE_PATH).expect("create archive");
    serde_json::to_writer_pretty(
        file,
        &serde_json::json!({
            "results": records,
            "thresholds": thresholds,
            "tree": tree,
        }),
    )
    .expect("write archive");
}
