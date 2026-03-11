use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::thread_rng;

use array_vis_bench::bench_registry;
use array_vis_bench::utils::array_gen::get_rand_arr;

/// Drop a sort from all future N levels if its average over PROBE_RUNS exceeds this.
const SLOW_THRESHOLD: Duration = Duration::from_millis(50);

/// Number of timed runs used to decide whether a sort is too slow for this N.
const PROBE_RUNS: usize = 10;

/// Criterion samples collected per (sort × N) pair that passes the threshold.
const SAMPLES: usize = 10;

const MAX_N: usize = 10_000_000;

fn bench_sorts(c: &mut Criterion) {
    let mut active: Vec<&'static bench_registry::SortBenchEntry> =
        bench_registry::BENCH_SORTS.iter().collect();

    let mut rng = thread_rng();
    let mut n = 10usize;

    while n <= MAX_N && !active.is_empty() {
        let mut buf = get_rand_arr(n);
        let mut too_slow: Vec<&'static str> = Vec::new();

        {
            let mut group = c.benchmark_group(format!("sorts/n={n}"));
            group.sample_size(SAMPLES);
            // The pre-check below acts as warmup; skip criterion's own warmup
            // so total time per (sort × N) stays close to 2 seconds.
            group.warm_up_time(Duration::from_millis(20));
            group.measurement_time(Duration::from_millis(1500));

            for entry in &active {
                // ------------------------------------------------------------------
                // Pre-check: PROBE_RUNS timed runs to compute the average sort time.
                // If the average exceeds SLOW_THRESHOLD the sort is dropped and we
                // skip the criterion benchmark entirely — no waiting for criterion
                // to laboriously time-out a slow sort across all samples.
                // ------------------------------------------------------------------
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

                // ------------------------------------------------------------------
                // Full criterion benchmark for sorts that pass the threshold.
                // iter_custom shuffles buf in-place before each timed sort so
                // there are zero per-iteration heap allocations from our side.
                // ------------------------------------------------------------------
                group.bench_function(entry.name, |b| {
                    b.iter_custom(|iters| {
                        let mut total = Duration::ZERO;
                        for _ in 0..iters {
                            buf.shuffle(&mut rng);
                            let t = Instant::now();
                            (entry.run)(&mut buf);
                            total += t.elapsed();
                        }
                        total
                    });
                });
            }
        } // group.finish() on drop

        drop(buf);

        active.retain(|e| !too_slow.contains(&e.name));

        if !too_slow.is_empty() {
            eprintln!(
                "n={n}: dropped {} sort(s): {}",
                too_slow.len(),
                too_slow.join(", ")
            );
        }

        n *= 8;
    }
}

criterion_group!(benches, bench_sorts);
criterion_main!(benches);
