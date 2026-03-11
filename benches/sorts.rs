use std::cell::Cell;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::thread_rng;

use array_vis_bench::bench_registry;
use array_vis_bench::utils::array_gen::get_rand_arr;

/// A sort taking longer than this for a single run is dropped from all larger N levels.
const SLOW_THRESHOLD: Duration = Duration::from_millis(100);

const ARRAYS_PER_N: usize = 10;
const RUNS_PER_ARRAY: usize = 10;

const MAX_N: usize = 10_000_000;

fn bench_sorts(c: &mut Criterion) {
    let mut active: Vec<&'static bench_registry::SortBenchEntry> =
        bench_registry::BENCH_SORTS.iter().collect();

    let mut rng = thread_rng();
    let mut n = 10usize;

    while n <= MAX_N && !active.is_empty() {
        // Allocate exactly n elements for this level.  For n >= ~16K glibc
        // uses mmap, so drop() returns the physical pages to the OS immediately.
        // This keeps peak RSS at 2×n×8 bytes (buf + sort's internal aux) rather
        // than holding MAX_N elements alive for the whole run.
        let mut buf = get_rand_arr(n);
        let mut too_slow: Vec<&'static str> = Vec::new();

        {
            let mut group = c.benchmark_group(format!("sorts/n={n}"));
            group.sample_size(ARRAYS_PER_N * RUNS_PER_ARRAY);
            group.warm_up_time(Duration::from_millis(500));
            group.measurement_time(Duration::from_secs(1));
            println!("n={n}: active sorts: {}, total {}", active.iter().map(|e| e.name).collect::<Vec<_>>().join(", "), active.len());

            for entry in &active {
                let was_slow = Cell::new(false);

                group.bench_function(entry.name, |b| {
                    b.iter_custom(|iters| {
                        let mut total = Duration::ZERO;
                        for _ in 0..iters {
                            buf.shuffle(&mut rng);

                            let start = Instant::now();
                            (entry.run)(&mut buf);
                            total += start.elapsed();
                        }
                        if total > SLOW_THRESHOLD {
                            was_slow.set(true);
                        }
                        total
                    });
                });

                if was_slow.get() {
                    too_slow.push(entry.name);
                }
            }
        } // group.finish() on drop

        drop(buf); // return physical pages to OS before next level

        active.retain(|e| !too_slow.contains(&e.name));

        if !too_slow.is_empty() {
            eprintln!(
                "n={n}: dropped {} slow sort(s): {}",
                too_slow.len(),
                too_slow.join(", ")
            );
        }

        n *= 8;
    }
}

criterion_group!(benches, bench_sorts);
criterion_main!(benches);
