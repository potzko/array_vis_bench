# benches

Criterion benchmark harness for comparing sort performance across array sizes.

## `sorts.rs`

Runs every sort registered in `BENCH_SORTS` (via `linkme` distributed slices) across doubling array sizes starting at N=10.

### Adaptive thresholding

Not all sorts scale the same. To avoid wasting time on O(N^2) sorts at large N, the harness uses a pre-check system:

1. Before each (sort, N) pair, run `PROBE_RUNS` (10) quick timed runs.
2. If the average exceeds `SLOW_THRESHOLD` (50ms), drop the sort from all remaining N levels.
3. Sorts that pass the threshold get a full Criterion benchmark with `SAMPLES` (10) samples.

This means the benchmark automatically focuses its time budget on the sorts that are competitive at each array size, while still including slow sorts at small N for completeness.

### Running

```bash
cargo bench --bench sorts
```

Results are written to `target/criterion/` in Criterion's standard format. Use `plot_bench.py` in the project root to generate comparison charts from the results.
