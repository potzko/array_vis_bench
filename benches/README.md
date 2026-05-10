# benches

Hand-rolled benchmark harness for comparing sort performance across array sizes.
Replaces criterion to keep per-(sort × N) overhead negligible.

## `sorts.rs`

Runs every sort registered in `BENCH_SORTS` (via `linkme` distributed slices) across doubling array sizes starting at N=10.

### Adaptive thresholding

Not all sorts scale the same. To avoid wasting time on O(N^2) sorts at large N:

1. Before each (sort, N) pair, run `PROBE_RUNS` (3) quick timed runs — these double as warmup.
2. If the average exceeds `SLOW_THRESHOLD` (50ms), drop the sort from all remaining N levels.
3. Sorts that pass the threshold get `SAMPLES` (10) timed measurements; mean and standard error are recorded.

### Running

```bash
cargo bench --bench sorts
python3 ../plot_bench.py
# open ../bench_report.html
```

Results are written to `target/bench_archive.json` as a flat list of records:

```json
{
  "results": [
    {"name": "...", "n": 10, "mean_ns": 1234.5, "stderr_ns": 12.3}
  ]
}
```

`plot_bench.py` reads that file directly and produces an interactive Plotly chart in `bench_report.html`.
