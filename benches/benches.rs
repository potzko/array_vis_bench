use std::time::Duration;

use array_vis_bench::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sorts::bubble_sorts::*;

use traits::SortAlgo;
use utils::array_gen::get_rand_arr;

fn bench_sorts_smallest_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_input_arrays");
    group.sample_size(10);
    group.measurement_time(core::time::Duration::from_millis(200));
    group.warm_up_time(Duration::from_millis(50));
    let funcs = [|arr: &mut [usize]| odd_even_bubble_sort::SortImp::sort(arr, &mut {})];
    let names = ["bubble_sort"];
    for (sort, name) in funcs.iter().zip(names) {
        for size in [5, 10, 15, 20, 25, 30, 35, 40, 45, 50] {
            let mut arr = get_rand_arr(size);
            group.bench_function(BenchmarkId::new(name, arr.len()), |b| {
                b.iter(|| sort(&mut arr))
            });
        }
    }
    group.finish();
}

fn bench_sorts_small_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_input_arrays");
    let funcs = [|arr: &mut [usize]| bubble_sort::SortImp::sort(arr, &mut {})];
    let names = ["bubble_sort"];
    for (sort, name) in funcs.iter().zip(names) {
        for size in [75, 100, 150, 200, 500] {
            let mut arr = get_rand_arr(size);
            group.bench_function(BenchmarkId::new(name, arr.len()), |b| {
                b.iter(|| sort(&mut arr))
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_sorts_smallest_random);
criterion_main!(benches);
