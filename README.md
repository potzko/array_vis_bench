# `array_vis_bench`

A Rust framework for composing algorithms over ordered data, then visualising,
checking and benchmarking every composition. Sorting is the first domain; the
machinery is domain agnostic.

The project stands on five ideas:

- Zero runtime cost. Every algorithm is written once, generic over a logger.
  The benchmark path uses a no-op logger that compiles away, so the measured
  code is the same code a hand written sort would produce.
- Elegant to develop. A new pivot selector, partition or small sort is one tiny
  crate plus a five line declaration. Nothing else is edited.
- Ordered data visualisation. The same code, run under a recording logger,
  yields an event stream the renderer turns into an MP4.
- Correctness. Every emitted variant carries its own correctness battery and
  runs under the workspace test suite.
- Composition. Algorithms are typed compositions of role fillers. A build time
  constraint compiler enumerates the legal compositions from declarative spec
  and query files and emits one registry entry per concrete type.

## The pipeline in one picture

```
leaf crates          .spec fragments       .avbs queries
(Rust impls)         (components, roles)   (which compositions to build)
      \                    |                     /
       \                   v                    /
        spec_catalog/build.rs: gather -> merge -> solve -> emit
                           |
                           v
            catalog_entries.rs: one AlgorithmEntry per concrete type
                           |
          +----------------+----------------+
          v                v                v
     visualiser       correctness       benchmark
   (MP4 render)     (per kind battery)  (timing tables)
```

A query names a driver and fills its slots with `_` (every filler of that
role), a set difference, a curated union, or a sampled subset. The solver
produces the ground types, rustc type checks them, and the emitted module for
each one registers an `AlgorithmEntry` into a linkme distributed slice.

## Current catalog

1588 entries are emitted from the build at the tip of this branch.

| family | entries | notes |
|---|---|---|
| quick sorts | 408 | 2 drivers x 51 partitions x 4 small sorts |
| heap sorts | 400 | 16 classic + 360 quick build + 24 dual build |
| merge sorts | 216 | 40 classic + 176 rotation based |
| fun sorts | 195 | slow, stooge, cyclent, quick surrender, random shell, bad heap |
| beap sorts | 80 | 2 classic + 72 quick build + 6 dual build |
| quick select | 56 | recursive and iterative, pivoted and heap extract |
| quick heap sorts | 34 | 16 classic + 16 deferred + 2 set quick select |
| shell sorts | 18 | 9 gap sequences x 2 drivers |
| circle sorts | 17 | |
| rod sorts | 12 | 6 branching strategies x 2 merges |
| bubble sorts | 6 | 3 base + 3 odd even |
| comb sorts | 6 | 6 shrink ratios |
| weak heap sorts | 4 | 2 directions x 2 bit storages |
| block merge sorts | 6 | 3 buffers x 2 key shapes, the WikiSort and GrailSort shapes among them |
| scratchspace nightmare sorts | 3 | 3 backings |
| insertion sorts | 2 | linear and binary |
| tree sorts | 2 | avl and naive trees over one node store |
| cycle sort | 1 | |
| first class kinds | 122 | 11 rotations, 7 small sorts, 67 partitions, 25 merges, 12 scratchspace workflows |

Every count is pinned by a golden test in `spec_catalog`, so a solver or
emit regression that drops a variant fails the build's own tests.

## Running it

| command | what it does |
|---|---|
| `cargo run --release` | Interactive visualiser. Walks category, family and facets, asks for an input shape, size and render settings, writes `output.mp4`. Needs `ffmpeg` on the path. |
| `cargo run --release -- --consumers` | Runs the consumer program declared in `spec_catalog/consumers.avbs` (correctness and benchmark) over the whole emitted catalog, the benchmark at its defaults, and prints one report per consumer. |
| `cargo run --release -- --timing [N] [K] [--input NAME] [--out PATH]` | Runs the benchmark consumer over the whole catalog at size N (default 1000) for K rounds (default 3) on each category's primary input, or on the named shape for the categories that have it. Each entry gets an unmeasured warm up, short runs are batched so a microsecond sort is not one tick of the clock, the input is built before the clock starts, and every round runs every entry once so a burst of load spreads. The table per category ranks medians fastest first with the inter quartile range and the Tukey outlier count beside each; `--out` writes those as the archive, which ends with one `# entry` line per entry giving its category and the role, value and path of each of its axes. An entry whose cap is below N, or whose category lacks the shape, is skipped and counted. |
| `cargo run --release -- --sweep SECONDS [--share K] [--only PREFIX] [--input NAME] [--max-size N] [--out PATH]` | The scaling sweep: every entry, or those whose name starts with PREFIX, climbs the size ladder (every size to 100, tens to 200, hundreds to 1000, then powers of two) until one run takes more than SECONDS / (entries * K), K defaulting to 5, or the size passes N, 4194304 by default, and the time left binary searches each entry's largest size that fits. The budget pays for building the inputs as well as the runs, and the table's header says what was spent; the climbs themselves are not cut short, so a small budget over the whole catalog spends more than it asks. Prints one table ranked by that size; `--out` writes the archive `plot_bench.py` charts, which ends with the same `# entry` lines. |
| `cargo run --release --bin compare_sorts` | Performance regression gate. Pits the generic quick, merge and heap sorts against hand rolled u64 baselines and asserts each pair stays within 1.05x. |
| `cargo run --release --bin measure_random_shell` | Complexity probe for the random shell sort variants. |
| `cargo run -p array_vis_bench_min --release` | Minimal link root with two families and no catalog, proving a consumer can take just the leaves it wants. |
| `./verify.sh [--quick \| --portable] [--bless]` | The verification gate: release build without warnings, the compiler's golden and the emitted catalog's types, the workspace tests, the assembly of the gate symbols, the `compare_sorts` gate and a timing comparison, each against a baseline under `verify/`. `--portable` runs only the steps that do not depend on this machine, through the workspace tests. A pass in a clean checkout stamps the tree for the branch gate, see [githooks/README.md](githooks/README.md). |
| `./promote.sh [--push] BRANCH \| main` | Fast forwards `dev` to BRANCH, or `main` to `dev`, once the source's tree carries the stamp the target needs, running the gate in the source's clean worktree first when it does not. `--push` pushes the target to origin. |
| `cargo t` | The whole workspace's tests under nextest in release: compiler unit tests, per crate unit tests, and every emitted entry's correctness battery. The battery runs each entry in a subprocess so a hang or crash names the variant. |

`cargo t` is an alias in `.cargo/config.toml` for `cargo nextest run --release
--workspace`; nextest reports per test where `cargo test` prints one block per
target, over a hundred here, nearly all empty. `.config/nextest.toml` pins the
tool version, and `cargo install cargo-nextest --locked` installs it. Plain
`cargo test --release` still works and is what a crate's own `cargo test -p`
runs.

Helper scripts:

- `asm_text.sh`, `dump_asm.sh` and `asmdiff.sh` list, dump and diff the
  release assembly of the hand rolled and generic sorts measured by
  `compare_sorts`, which is how the zero cost claim is checked.
- `verify.sh` runs every check above against the baselines in `verify/` and
  rewrites them with `--bless` after an intended change.
- `plot_bench.py` writes the benchmark page from timing and sweep
  archives, `--timing N K [--input NAME] --out PATH` or `--sweep SECONDS
  --out PATH`, any mix of sizes and input shapes. The sidebar has a
  checklist per kind, family and axis with the count each value would bring
  back. The page shows time against size with a panel per input shape,
  coloured by any facet, collapsed to the best of a facet or cut to the
  entries no other dominates over a size range; the effect of a facet
  against the best value in its context; and parallel coordinates. The
  view lives in the URL, so a link reopens it. See
  [bench_view/README.md](bench_view/README.md).

## Branches

Work happens on a branch off `dev`, usually in a worktree under
`.claude/worktrees/`, and lands on `dev` as a fast forward by
`./promote.sh BRANCH`. `dev` takes a tree that passed
`./verify.sh --portable`, which runs anywhere; `main` takes one that passed
the full `./verify.sh` on the machine the baselines were taken on, and moves
only by `./promote.sh main`. The pre-push hook in
[githooks/](githooks/README.md) refuses a push of either branch whose tree
lacks that stamp.

## Workspace layout

```
.                        the default binary (spec only link root) + src/bin tools
array_vis_bench_cli      the interactive picker, prompts and render call
array_vis_bench_core     the ABI: AlgorithmEntry, ALGORITHMS, inputs, per kind
                         run drivers and correctness batteries, Main consumers
array_vis_bench_traits   role traits (PartitionScheme, PivotSelector, SmallSort,
                         Rotation, QuickSelect, Auxiliary, Keys, Scratchspace), the
                         composable annotations and the Complexity algebra
array_vis_bench_min      minimal link root demo
sort_logger              SortLog<T> events, SortLogger<T> trait, NoOpLogger,
                         VisualizerLogger
sort_vis                 MP4 renderer over a SortLog stream (drives ffmpeg)

spec_core                the constraint compiler: registry, query lowering,
                         solver, cardinality guard, emit scaffold
spec_emit                the array domain emit backend (AlgorithmEntry bodies)
spec_catalog             the unified catalog: gathers every crate's .spec and
                         .avbs through cargo metadata, solves, emits, and
                         pins the golden counts

crates/families/*        drivers and composers, one crate per sort family
crates/partitions/*      one crate per partition scheme
crates/pivots/*          one crate per pivot selector
crates/rotations/*       one crate per rotation
crates/small_sorts/*     small sort strategies and networks
crates/components/*      shared components (scratchspace, gap distributions,
                         reverse bit storage)

docs/                    architecture, catalog pipeline, trait system, how to
                         add a component, implementation plans, LLM guides
```

Every folder carries a README saying what it holds.

## How an algorithm enters the catalog

1. Implement a role trait in a leaf crate, plus the four annotation traits
   that give the type its complexity and stability.
2. Declare the type once in the crate's `.spec` file: a component name, the
   Rust type with `{slot}` holes, a label, the roles it provides.
3. Point the manifest at the fragment with `[package.metadata.array_vis_bench]`
   and make sure `spec_catalog` depends on the crate.
4. If the crate owns a driver, write its `.avbs` query. Leaves usually need
   none: the drivers' queries pick them up through their role.

The build re-derives every cross product. There is no central list of
algorithm names anywhere. See [docs/writing-components.md](docs/writing-components.md)
for the full guide and a worked example.

## Logging and visualisation

`SortLogger<T>` is the abstraction every algorithm writes to. Every comparison,
swap, write and auxiliary allocation goes through it. Benchmarks and tests use
`NoOpLogger`, whose methods compile away. The visualiser swaps in
`VisualizerLogger<T>`, captures the event stream, and hands it to the renderer.
The trait is dyn compatible, so the registry's function pointers take
`&mut dyn SortLogger<usize>` while the algorithm bodies stay monomorphic.

## Status and plans

[docs/todo.md](docs/todo.md) records what the compiler migration delivered
and what is still open. Designed but unbuilt work lives in
[docs/implementation_plans/](docs/implementation_plans/).

## Note for LLM assistants

Read [docs/llm/README.md](docs/llm/README.md) first. It says which files to
load, the patterns to follow, and how to keep the documentation in step with
the code.
