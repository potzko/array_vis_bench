use std::fs::File;
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;

use array_vis_bench::bench_registry;
use array_vis_bench::utils::array_gen::get_rand_arr;

/// Drop a sort from all future N levels if its average over PROBE_RUNS exceeds this.
const SLOW_THRESHOLD: Duration = Duration::from_millis(50);

/// Number of timed runs used to decide whether a sort is too slow for this N.
/// Also serves as the warmup before the sample loop.
const PROBE_RUNS: usize = 3;

/// Timed runs collected per (sort × N) pair that passes the threshold.
const SAMPLES: usize = 10;

const MAX_N: usize = 5_000_000_000;

const ARCHIVE_PATH: &str = "target/bench_archive.json";

#[derive(Serialize)]
struct Record {
    name: &'static str,
    n: usize,
    mean_ns: f64,
    stderr_ns: f64,
}

fn main() {
    let mut active: Vec<&'static bench_registry::SortBenchEntry> =
        bench_registry::BENCH_SORTS.iter().collect();

    let mut rng = thread_rng();
    let mut n = 10usize;
    let mut records: Vec<Record> = Vec::new();

    while n <= MAX_N && !active.is_empty() {
        let mut buf = get_rand_arr(n);
        let mut too_slow: Vec<&'static str> = Vec::new();

        for entry in &active {
            // Probe doubles as warmup — drops sorts whose average exceeds the threshold.
            let mut probe_total = Duration::ZERO;
            for _ in 0..PROBE_RUNS {
                buf.shuffle(&mut rng);
                let t = Instant::now();
                (entry.run)(&mut buf);
                probe_total += t.elapsed();
            }
            if probe_total / PROBE_RUNS as u32 > SLOW_THRESHOLD {
                too_slow.push(entry.name);
                continue;
            }

            // Shuffle is outside the timed window so it doesn't pollute the measurement.
            let mut samples = [0u64; SAMPLES];
            for s in &mut samples {
                buf.shuffle(&mut rng);
                let t = Instant::now();
                (entry.run)(&mut buf);
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
        }

        drop(buf);
        active.retain(|e| !too_slow.contains(&e.name));

        if !too_slow.is_empty() {
            eprintln!(
                "n={n}: dropped {} sort(s): {}",
                too_slow.len(),
                too_slow.join(", ")
            );
        }

        // Persist after every N level so a Ctrl-C still leaves useful data.
        write_archive(&records);

        n *= 2;
    }

    println!("\nWrote {} records to {}", records.len(), ARCHIVE_PATH);
}

fn write_archive(records: &[Record]) {
    std::fs::create_dir_all("target").ok();
    let file = File::create(ARCHIVE_PATH).expect("create archive");
    serde_json::to_writer_pretty(file, &serde_json::json!({ "results": records }))
        .expect("write archive");
}
