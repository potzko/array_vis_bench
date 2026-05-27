//! Algorithm + input registry — the runtime spine of the workspace.
//!
//! Every algorithm leaf (sorts, rotations, partitions, merges,
//! small-sorts, quick-selects) registers itself into [`ALGORITHMS`] via
//! `#[linkme::distributed_slice]` at link time. The harness — bench,
//! visualiser, correctness tests — sees one type-erased
//! [`AlgorithmEntry`] regardless of category, and dispatches through
//! the two fn pointers each entry carries:
//!
//! - `run_with_input` — drives the algorithm with a dyn-logger.
//! - `run_correctness` — drives the category's correctness battery.
//!
//! Input shapes are registered separately, per category, into a
//! distributed slice (`SORT_INPUTS`, `ROTATION_INPUTS`, etc.). The
//! algorithm picks its input by name at runtime; the input
//! implements its category's `*Input` trait to translate a
//! [`RunConfig`] into the algorithm's natural-shape argument list.
//!
//! Library consumers usually only read `ALGORITHMS` (and friends);
//! producers of new algorithms register through the `sort_family!` /
//! `register_input!` macros and never touch the registry directly.

use linkme::distributed_slice;

// ── Generic algorithm registry ───────────────────────────────────────────────

/// What kind of algorithm an [`AlgorithmEntry`] describes.
///
/// The harness layer (bench, vis, tests) is category-agnostic — this
/// enum exists for menu grouping and to let the per-category
/// correctness batteries pick the right verification logic.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Category {
    /// A full sort. Input: `Vec<usize>`. Postcondition: sorted ascending.
    Sort,
    /// A slice rotation. Input: `(Vec<usize>, split)`. Postcondition:
    /// elements at `[split..]` are now at the front.
    Rotation,
    /// One partition step around a pivot. Input: `Vec<usize>`. Output:
    /// `(left_end, right_start)` boundary indices.
    Partition,
    /// A single merge of two adjacent sorted runs. Input:
    /// `(Vec<usize>, mid)`. Postcondition: sorted ascending.
    Merge,
    /// A small-input fast path (size threshold typically ≤ 32).
    SmallSort,
    /// k-th-order-statistic finder. Input: `(Vec<usize>, target)`.
    /// Postcondition: the element that would land at `target` after a
    /// full sort is at `arr[target]`.
    QuickSelect,
}

impl Category {
    /// Lowercase short name (used in menu paths and error messages).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Sort => "sort",
            Category::Rotation => "rotation",
            Category::Partition => "partition",
            Category::Merge => "merge",
            Category::SmallSort => "small-sort",
            Category::QuickSelect => "quick-select",
        }
    }
}

/// Tunable knobs for a single run. Each category's input list interprets
/// these knobs as it pleases — most use just `size` and `seed`.
#[derive(Clone, Debug)]
pub struct RunConfig {
    pub size: usize,
    pub seed: u64,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig { size: 500, seed: 0 }
    }
}

// ── Per-category input traits ────────────────────────────────────────────────
//
// Each category has one trait that registered input types implement. The
// trait's only job is `generate` — translate a `RunConfig` into the
// natural-shape input the algorithm wants. Concrete `*Input` types
// register themselves into the matching distributed slice via the
// `register_input!` macros below, so they're selectable by name at
// runtime without giving up the typed `generate` signature.

/// Generates the input array for a sort benchmark / visualisation.
pub trait SortInput {
    /// Build a `Vec<usize>` of length `config.size` following this
    /// input's shape (random / sorted / reversed / sawtooth / …).
    fn generate(config: &RunConfig) -> Vec<usize>;
}

/// Generates the input for a single rotation step.
pub trait RotationInput {
    /// Returns (array, split index).
    fn generate(config: &RunConfig) -> (Vec<usize>, usize);
}

/// Generates the input for a single merge step — two adjacent runs.
pub trait MergeInput {
    /// Returns (array, mid). `arr[..mid]` and `arr[mid..]` are each sorted.
    fn generate(config: &RunConfig) -> (Vec<usize>, usize);
}

/// Generates the input for a small-sort variant. Same shape as
/// [`SortInput`] but typically sized below the algorithm's threshold.
pub trait SmallSortInput {
    /// Build a `Vec<usize>` shorter than the small-sort threshold.
    fn generate(config: &RunConfig) -> Vec<usize>;
}

/// Generates the input for a quickselect step.
pub trait QuickSelectInput {
    /// Returns (array, target index). Quickselect reorders `arr` so
    /// that the value which would land at `target` after a full sort
    /// lands there. Different target positions stress different
    /// recursion paths (mid = balanced, first = always recurse left,
    /// last = always recurse right).
    fn generate(config: &RunConfig) -> (Vec<usize>, usize);
}

// ── Per-category input registries ────────────────────────────────────────────
//
// Inputs are first-class registered things, on par with algorithms. Each
// category gets a distributed slice keyed by the natural-shape signature
// of its `generate` fn pointer. UI / harness code iterates the slice for
// the algorithm's category to populate input pickers / lookups by name.

pub struct SortInputEntry {
    pub name: &'static str,
    pub generate: fn(&RunConfig) -> Vec<usize>,
    /// Exactly one entry per category is `primary = true`. Harness code
    /// that needs a sensible default without prompting the user picks
    /// the primary. Multiple primaries → startup panic; zero primaries
    /// → also a startup panic.
    pub primary: bool,
}

pub struct RotationInputEntry {
    pub name: &'static str,
    pub generate: fn(&RunConfig) -> (Vec<usize>, usize),
    pub primary: bool,
}

pub struct MergeInputEntry {
    pub name: &'static str,
    pub generate: fn(&RunConfig) -> (Vec<usize>, usize),
    pub primary: bool,
}

pub struct SmallSortInputEntry {
    pub name: &'static str,
    pub generate: fn(&RunConfig) -> Vec<usize>,
    pub primary: bool,
}

pub struct QuickSelectInputEntry {
    pub name: &'static str,
    pub generate: fn(&RunConfig) -> (Vec<usize>, usize),
    pub primary: bool,
}

#[distributed_slice] pub static SORT_INPUTS:         [SortInputEntry] = [..];
#[distributed_slice] pub static ROTATION_INPUTS:     [RotationInputEntry] = [..];
#[distributed_slice] pub static MERGE_INPUTS:        [MergeInputEntry] = [..];
#[distributed_slice] pub static SMALL_SORT_INPUTS:   [SmallSortInputEntry] = [..];
#[distributed_slice] pub static QUICK_SELECT_INPUTS: [QuickSelectInputEntry] = [..];

/// All registered input names for `category`, in registry order.
///
/// `Partition` shares the `Sort` input registry: a partition algorithm
/// (with internal pivot selection) takes the same shape of input as a
/// sort — a single `Vec<usize>` — so any sort input is meaningful for
/// it. No need for a parallel partition-input registry.
pub fn list_inputs(category: Category) -> Vec<&'static str> {
    match category {
        Category::Sort | Category::Partition => SORT_INPUTS.iter().map(|e| e.name).collect(),
        Category::Rotation => ROTATION_INPUTS.iter().map(|e| e.name).collect(),
        Category::Merge => MERGE_INPUTS.iter().map(|e| e.name).collect(),
        Category::SmallSort => SMALL_SORT_INPUTS.iter().map(|e| e.name).collect(),
        Category::QuickSelect => QUICK_SELECT_INPUTS.iter().map(|e| e.name).collect(),
    }
}

/// The primary input for `category` — the one harness code uses as a
/// sensible default when there's no explicit user pick. Each category
/// must have exactly one primary; uniqueness is enforced at startup by
/// [`validate_registries`].
pub fn primary_input(category: Category) -> &'static str {
    match category {
        Category::Sort | Category::Partition => {
            SORT_INPUTS.iter().find(|e| e.primary).map(|e| e.name)
        }
        Category::Rotation => ROTATION_INPUTS.iter().find(|e| e.primary).map(|e| e.name),
        Category::Merge => MERGE_INPUTS.iter().find(|e| e.primary).map(|e| e.name),
        Category::SmallSort => SMALL_SORT_INPUTS.iter().find(|e| e.primary).map(|e| e.name),
        Category::QuickSelect => QUICK_SELECT_INPUTS.iter().find(|e| e.primary).map(|e| e.name),
    }
    .unwrap_or_else(|| panic!("no primary input registered for category {:?}", category))
}

/// Validate the registries at startup. Runs once via [`#[ctor::ctor]`]
/// (see `validate_at_startup` below). The checks are deliberately strict
/// so any drift — a duplicated algorithm name, two primary inputs in the
/// same category, an empty input registry for a category that has
/// algorithm entries — fails fast with a precise message at process
/// start instead of confusing the user later.
pub fn validate_registries() -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();

    // ── Algorithm name uniqueness (per-category scope) ───────────────
    //
    // Names live in a category-scoped namespace: a sort and a partition
    // can both be called "left-left pointer" without colliding (in practice they
    // won't because the rotation/partition/small-sort macros bake a
    // category-prefix into the name anyway, but the check enforces it
    // independent of that convention). Two algorithms in the *same*
    // category with the same name → hard error.
    {
        let mut seen_per_cat: std::collections::HashMap<
            (Category, &'static str),
            usize,
        > = Default::default();
        for entry in ALGORITHMS {
            *seen_per_cat.entry((entry.category, entry.name)).or_insert(0) += 1;
        }
        for ((cat, name), count) in &seen_per_cat {
            if *count > 1 {
                errors.push(format!(
                    "duplicate algorithm name '{}' in category {:?} ({} registrations)",
                    name, cat, count
                ));
            }
        }
    }

    // ── Tree path uniqueness ─────────────────────────────────────────
    //
    // The interactive picker walks `sort_registry_core`'s tree and
    // expects one entry per algorithm. If the same name registers a
    // path twice (e.g. two ctors emitting the same family! call), the
    // tree picks one arbitrarily and the duplicate becomes invisible.
    // Catch that here rather than silently shipping a half-broken menu.
    {
        let entries = sort_registry_core::registered_path_entries();
        let mut by_name: std::collections::HashMap<String, Vec<Vec<String>>> =
            Default::default();
        for (name, path) in entries {
            by_name.entry(name).or_default().push(path);
        }
        for (name, paths) in &by_name {
            if paths.len() > 1 {
                let formatted: Vec<String> =
                    paths.iter().map(|p| format!("[{}]", p.join("/"))).collect();
                errors.push(format!(
                    "algorithm '{}' registered at {} distinct tree paths: {}",
                    name,
                    paths.len(),
                    formatted.join(", ")
                ));
            }
        }
    }

    // ── Per-category input checks ────────────────────────────────────
    fn check_inputs<E, NF, PF>(
        category: Category,
        entries: &[E],
        name_of: NF,
        primary_of: PF,
        algorithms_in_category: usize,
        errors: &mut Vec<String>,
    ) where
        NF: Fn(&E) -> &'static str,
        PF: Fn(&E) -> bool,
    {
        let mut seen: std::collections::HashSet<&'static str> = Default::default();
        for e in entries {
            if !seen.insert(name_of(e)) {
                errors.push(format!(
                    "duplicate input name '{}' in category {:?}",
                    name_of(e), category
                ));
            }
        }
        let primaries: Vec<&'static str> =
            entries.iter().filter(|e| primary_of(e)).map(&name_of).collect();
        if entries.is_empty() && algorithms_in_category > 0 {
            errors.push(format!(
                "no inputs registered for category {:?} (but {} algorithm(s) need one)",
                category, algorithms_in_category
            ));
        } else if !entries.is_empty() {
            match primaries.len() {
                0 => errors.push(format!(
                    "no primary input for category {:?}; mark one entry with `primary: true`",
                    category
                )),
                1 => {}
                _ => errors.push(format!(
                    "multiple primary inputs for category {:?}: {:?}",
                    category, primaries
                )),
            }
        }
    }

    // Partitions reuse SORT_INPUTS, so they're not validated as a
    // separate registry. The "sort inputs exist when sort or partition
    // algorithms exist" check rolls partition count into the sort
    // category's expected-non-empty assertion.
    let sort_or_partition_count =
        ALGORITHMS.iter().filter(|e| matches!(e.category, Category::Sort | Category::Partition)).count();
    let count = |cat: Category| ALGORITHMS.iter().filter(|e| e.category == cat).count();
    let sort_inputs: &[SortInputEntry] = &SORT_INPUTS;
    let rotation_inputs: &[RotationInputEntry] = &ROTATION_INPUTS;
    let merge_inputs: &[MergeInputEntry] = &MERGE_INPUTS;
    let small_inputs: &[SmallSortInputEntry] = &SMALL_SORT_INPUTS;
    let qsel_inputs: &[QuickSelectInputEntry] = &QUICK_SELECT_INPUTS;
    check_inputs(Category::Sort,        sort_inputs,     |e| e.name, |e| e.primary, sort_or_partition_count,       &mut errors);
    check_inputs(Category::Rotation,    rotation_inputs, |e| e.name, |e| e.primary, count(Category::Rotation),     &mut errors);
    check_inputs(Category::Merge,       merge_inputs,    |e| e.name, |e| e.primary, count(Category::Merge),        &mut errors);
    check_inputs(Category::SmallSort,   small_inputs,    |e| e.name, |e| e.primary, count(Category::SmallSort),    &mut errors);
    check_inputs(Category::QuickSelect, qsel_inputs,     |e| e.name, |e| e.primary, count(Category::QuickSelect),  &mut errors);

    if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
}

/// Ctor-style early-init that runs [`validate_registries`] before
/// anything else. A registry-shape error here panics at process start —
/// no test, viz, or bench gets to observe the inconsistent state.
#[ctor::ctor]
fn validate_at_startup() {
    if let Err(msg) = validate_registries() {
        // Skip validation when the binary was re-execed as the
        // subprocess test runner — that ctor (`subprocess_dispatch`)
        // runs after this one and has its own exit path. Panicking
        // here would mask its more-useful error message.
        if std::env::var(SUBPROCESS_ENV_VAR).is_ok() {
            return;
        }
        panic!("bench_registry validation failed:\n{msg}");
    }
}

// ── Shared per-category dispatchers ──────────────────────────────────────────
//
// The macro-generated `run_with_input` in every algorithm entry is a
// one-liner around the matching helper below. Putting the
// init-event-emitting code in a single place keeps the generated code
// small and centralises the visualiser contract ("CreateArr + N
// WriteData fully describes the initial state").

/// Run a sort against a named input. Looks the input up in
/// [`SORT_INPUTS`], generates the array via the registered `generate`
/// fn, emits the initial-state events on `logger`, then calls
/// `sort_fn(arr, logger)`.
pub fn run_sort_with_input(
    input_name: &str,
    config: &RunConfig,
    sort_fn: fn(&mut [usize], &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = SORT_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("sort input '{input_name}' not registered"));
    let mut arr = (input.generate)(config);
    emit_init_events(&arr, logger);
    sort_fn(&mut arr, logger);
}

/// Run a rotation against a named input. Returns the (arr, split) tuple
/// from the input registry; rotates with `rotate_fn(arr, split, logger)`.
pub fn run_rotation_with_input(
    input_name: &str,
    config: &RunConfig,
    rotate_fn: fn(&mut [usize], usize, &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = ROTATION_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("rotation input '{input_name}' not registered"));
    let (mut arr, split) = (input.generate)(config);
    emit_init_events(&arr, logger);
    rotate_fn(&mut arr, split, logger);
}

/// Run a partition against a named input. Partitions register with
/// their pivot selector baked in, so the standalone partition fn
/// signature matches a sort's — `(&mut [usize], &mut dyn SortLogger)`
/// — and the input registry is just `SORT_INPUTS`. The (left_end,
/// right_start) tuple a partition returns isn't useful to the
/// visualiser; tests use the `PartitionFnPtr` variant that keeps the
/// return value.
pub fn run_partition_with_input(
    input_name: &str,
    config: &RunConfig,
    partition_fn: fn(&mut [usize], &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = SORT_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("sort input '{input_name}' not registered"));
    let mut arr = (input.generate)(config);
    emit_init_events(&arr, logger);
    partition_fn(&mut arr, logger);
}

/// Run a merge against a named input. The registry guarantees both
/// halves of `arr` are individually sorted before merge.
pub fn run_merge_with_input(
    input_name: &str,
    config: &RunConfig,
    merge_fn: fn(&mut [usize], usize, &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = MERGE_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("merge input '{input_name}' not registered"));
    let (mut arr, mid) = (input.generate)(config);
    emit_init_events(&arr, logger);
    merge_fn(&mut arr, mid, logger);
}

/// Run a quick-select against a named input. The input supplies both
/// the array and the target index; the algorithm reorders the array so
/// `arr[target]` ends up as the value it would have after a full sort.
pub fn run_quick_select_with_input(
    input_name: &str,
    config: &RunConfig,
    quick_select_fn: fn(&mut [usize], usize, &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = QUICK_SELECT_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("quick-select input '{input_name}' not registered"));
    let (mut arr, target) = (input.generate)(config);
    emit_init_events(&arr, logger);
    quick_select_fn(&mut arr, target, logger);
}

/// Run a small-sort against a named input. Mirrors `run_sort_with_input`
/// but pulls from [`SMALL_SORT_INPUTS`].
pub fn run_small_sort_with_input(
    input_name: &str,
    config: &RunConfig,
    sort_fn: fn(&mut [usize], &mut dyn sort_logger::SortLogger<usize>),
    logger: &mut dyn sort_logger::SortLogger<usize>,
) {
    let input = SMALL_SORT_INPUTS
        .iter()
        .find(|e| e.name == input_name)
        .unwrap_or_else(|| panic!("small-sort input '{input_name}' not registered"));
    let mut arr = (input.generate)(config);
    emit_init_events(&arr, logger);
    sort_fn(&mut arr, logger);
}

/// Emit the visualiser's "this array exists with these initial values"
/// event sequence: one CreateAuxArrT + one WriteData per element. The
/// visualiser derives each array's bar-height scale from a pre-pass over
/// the log, so no scale hint is needed here. The log alone fully
/// describes the initial state.
fn emit_init_events(arr: &[usize], logger: &mut dyn sort_logger::SortLogger<usize>) {
    logger.log_aux_arr_t(arr);
    let name = arr.as_ptr() as usize;
    for (i, &v) in arr.iter().enumerate() {
        logger.log(sort_logger::SortLog::WriteData { name, ind: i, data: v });
    }
}

pub struct AlgorithmEntry {
    pub name: &'static str,
    pub category: Category,
    /// Worst-case time complexity (Big-O). Sites use either
    /// `Complexity::from_str("O(...)")` for legacy literal annotations or
    /// `<ConcreteType as HasTimeBounds>::WORST` when generated from the
    /// trait machinery via `big_o = inherited`.
    pub worst: array_vis_bench_traits::Complexity,
    /// Best-case time complexity (Big-Omega). Defaults to `worst` for
    /// algorithms that haven't been split into separate WORST/BEST/AVERAGE
    /// via `HasTimeBounds`.
    pub best: array_vis_bench_traits::Complexity,
    /// Average-case time complexity (Big-Theta when tight). Defaults to
    /// `worst` for legacy entries.
    pub average: array_vis_bench_traits::Complexity,
    /// Auxiliary-space complexity (heap allocations that grow with N).
    /// Bounded-stack buffers and recursion depth count as `O(log N)` or
    /// less — see `Complexity::is_in_place()` for the `in_place` derivation.
    pub space: array_vis_bench_traits::Complexity,
    /// Sort-relevant flag; ignored for non-sort categories.
    pub stable: bool,
    /// True if the algorithm runs faster on nearly-sorted input
    /// (e.g. insertion sort, Tim sort). Not compositional — declared
    /// per-family as a literal.
    pub adaptive: bool,
    /// Optional contract-defined upper bound on input size. `None` =
    /// unbounded (every general-purpose sort/rotation/partition/merge).
    /// `Some(n)` means inputs larger than `n` are out-of-contract —
    /// e.g. a small-sort with `THRESHOLD = 32`. The interactive picker
    /// uses this to cap the size prompt; `run_with_input` does its own
    /// defensive clamp so misuse from elsewhere is still safe.
    pub max_input_size: Option<usize>,
    /// Run the algorithm against the named input. The input must be
    /// registered in the input registry matching `category`. The
    /// algorithm emits all events (including CreateArr + N WriteData
    /// for the input) on `logger` — the log alone fully describes the
    /// visualisation.
    pub run_with_input:
        fn(input_name: &str, config: &RunConfig, &mut dyn sort_logger::SortLogger<usize>),
    /// Run the category-appropriate correctness battery (multiple inputs
    /// + verifier per input). Panics on failure. Uses `NoOpLogger`
    /// internally; never emits visualiser events.
    pub run_correctness: fn(),
}

#[distributed_slice]
pub static ALGORITHMS: [AlgorithmEntry] = [..];

/// Opt-in cap registry: `(sort_name, max_n_for_random_inputs)` pairs.
/// Sorts that can't handle large random inputs in reasonable time add
/// themselves via `register_test_cap!` (or a manual `distributed_slice`
/// entry); `max_n_for_tests` looks the cap up by name. Default is
/// "no cap" — no change needed at the call site for fast sorts.
#[distributed_slice]
pub static SORT_TEST_CAPS: [(&'static str, usize)] = [..];

/// Opt-out registry for algorithms whose `SortLog` trace is intentionally
/// nondeterministic between runs. The determinism check in
/// `crate::property_tests::determinism` skips any entry whose `name`
/// appears here.
///
/// The default is "deterministic" — only algorithms that genuinely use
/// uninitialised entropy (e.g. randomised gap sequences) opt out via
/// `register_nondeterministic!`.
#[distributed_slice]
pub static NONDETERMINISTIC_ALGOS: [&'static str] = [..];

/// True if `name` was registered as nondeterministic via
/// `register_nondeterministic!`.
pub fn is_nondeterministic(name: &str) -> bool {
    NONDETERMINISTIC_ALGOS.iter().any(|n| *n == name)
}

/// Mark an algorithm as having a nondeterministic `SortLog` trace, so the
/// determinism check skips it. Place next to the algorithm's `family!`
/// invocation — one call per registered leaf name.
///
/// ```text
/// register_nondeterministic!("random shell sort<uniform>");
/// register_nondeterministic!("random shell sort<parabolic>");
/// ```
#[macro_export]
macro_rules! register_nondeterministic {
    ($name:expr) => {
        const _: () = {
            #[::linkme::distributed_slice($crate::bench_registry::NONDETERMINISTIC_ALGOS)]
            #[allow(non_upper_case_globals)]
            static ENTRY: &'static str = $name;
        };
    };
}

/// Return the random-input size cap declared for `sort_name`, if any.
/// Used by `correctness::check_sort` to skip oversized random arrays
/// for slow sorts.
pub fn max_n_for_tests(sort_name: &str) -> Option<usize> {
    SORT_TEST_CAPS
        .iter()
        .find(|(name, _)| *name == sort_name)
        .map(|(_, cap)| *cap)
}

/// Declare a random-input size cap for a sort. Place near the sort's
/// `family!` invocation:
///
/// ```text
/// register_test_cap!("bad heap sort", 1000);
/// ```
#[macro_export]
macro_rules! register_test_cap {
    ($name:expr, $cap:expr) => {
        const _: () = {
            #[::linkme::distributed_slice($crate::bench_registry::SORT_TEST_CAPS)]
            #[allow(non_upper_case_globals)]
            static CAP: (&'static str, usize) = ($name, $cap);
        };
    };
}

/// All registered algorithm entries in canonical menu order — depth-first
/// traversal of the registry's tree, which sorts each level by subtree
/// size so specialised (small-group) entries surface first.
///
/// `linkme` makes no guarantee about link-time ordering, so consumers
/// that produce user-visible output should iterate this instead of
/// `ALGORITHMS` directly. Bench output and UI menu therefore surface
/// variants in the same order without either side having to declare it.
pub fn sorted() -> Vec<&'static AlgorithmEntry> {
    let order: std::collections::HashMap<String, usize> =
        sort_registry_core::get_registered_sorts()
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect();
    let mut v: Vec<&'static AlgorithmEntry> = ALGORITHMS.iter().collect();
    v.sort_by_key(|e| (order.get(e.name).copied().unwrap_or(usize::MAX), e.name));
    v
}

pub fn for_each<F: FnMut(&'static AlgorithmEntry)>(mut f: F) {
    for entry in sorted() {
        f(entry);
    }
}

/// Environment variable name used to put a subprocess into
/// "run correctness battery for one algorithm and exit" mode.
pub const SUBPROCESS_ENV_VAR: &str = "AVB_RUN_CHECK_SORT";

/// Ctor-style early-init that hijacks the process before libtest's main
/// runs. When the parent sets `AVB_RUN_CHECK_SORT=<algorithm name>` and
/// re-execs the same binary, the child enters here, looks the algorithm
/// up in `ALGORITHMS`, runs its category battery via `run_correctness`,
/// and exits — never reaching the test runner at all. The subprocess
/// always shares the parent's exact build, so a freshly-added algorithm
/// is immediately available without rebuilding a separate runner.
#[ctor::ctor]
fn subprocess_dispatch() {
    let Ok(name) = std::env::var(SUBPROCESS_ENV_VAR) else { return };
    let entry = ALGORITHMS.iter().find(|e| e.name == name).unwrap_or_else(|| {
        eprintln!("algorithm not registered: {name}");
        std::process::exit(2);
    });
    (entry.run_correctness)();
    std::process::exit(0);
}

// ── Property-test integration hooks ──────────────────────────────────────────
//
// The fixed-pattern correctness batteries used to call into
// `crate::property_tests::*::run` directly under `#[cfg(test)]`. With
// the batteries living in this crate, that direct reference would only
// fire when `cargo test -p array_vis_bench_core` is run — useless,
// because the algorithm registry is populated by leaves linked into the
// `array_vis_bench` binary, not core's standalone test build.
//
// Instead, each category exposes a distributed slice of hook fn
// pointers. The wiring crate's test target contributes a hook entry
// (its `property_tests` module is the natural home); the batteries
// iterate the slice and call every registered hook. With no hooks
// registered (a non-test build), the slice is empty and the iteration
// is a no-op — preserving the existing "property tests only run under
// cargo test" contract.

pub type SortPropHook = fn(correctness::SortFnPtr, &str);
pub type RotationPropHook = fn(correctness::RotationFnPtr, &str);
pub type PartitionPropHook = fn(correctness::PartitionFnPtr, &str);
pub type MergePropHook = fn(correctness::MergeFnPtr, &str);
pub type QuickSelectPropHook = fn(correctness::QuickSelectFnPtr, &str);
pub type SmallSortPropHook = fn(correctness::SmallSortFnPtr, &str, usize);

#[distributed_slice] pub static SORT_PROP_HOOKS:         [SortPropHook] = [..];
#[distributed_slice] pub static ROTATION_PROP_HOOKS:     [RotationPropHook] = [..];
#[distributed_slice] pub static PARTITION_PROP_HOOKS:    [PartitionPropHook] = [..];
#[distributed_slice] pub static MERGE_PROP_HOOKS:        [MergePropHook] = [..];
#[distributed_slice] pub static QUICK_SELECT_PROP_HOOKS: [QuickSelectPropHook] = [..];
#[distributed_slice] pub static SMALL_SORT_PROP_HOOKS:   [SmallSortPropHook] = [..];

/// Correctness-test runner. Public so the subprocess dispatch above can
/// call it; not `#[cfg(test)]` for the same reason.
///
/// Each category exposes one generic *battery* function that takes the
/// algorithm's natural-signature fn pointer and the algorithm's name.
/// The per-leaf `run_correctness` emitted by every `*_family!` macro
/// calls into the matching battery, so the test logic is written once
/// per category rather than inlined per leaf.
pub mod correctness {
    use super::{
        max_n_for_tests, MERGE_PROP_HOOKS, PARTITION_PROP_HOOKS, QUICK_SELECT_PROP_HOOKS,
        ROTATION_PROP_HOOKS, SMALL_SORT_PROP_HOOKS, SORT_PROP_HOOKS,
    };
    use crate::array_gen::{get_arr, get_rand_arr, get_rand_arr_in_range, get_reversed_arr};
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use rand::Rng;
    use sort_logger::NoOpLogger;

    /// Type alias for the natural-signature sort entry the macros emit
    /// (`<Sort>::sort(arr, logger)` constrained to `usize` + `NoOpLogger`
    /// for the test path).
    pub type SortFnPtr = fn(&mut [usize], &mut NoOpLogger);

    pub type RotationFnPtr = fn(&mut [usize], usize, &mut NoOpLogger);
    /// Standalone-partition fn pointer for tests. The pivot selector
    /// is baked into the registered partition, so callers don't pass
    /// one — but the (left_end, right_start) tuple is still returned
    /// so the battery can verify the partition contract.
    pub type PartitionFnPtr = fn(&mut [usize], &mut NoOpLogger) -> (usize, usize);
    pub type MergeFnPtr = fn(&mut [usize], usize, &mut NoOpLogger);
    pub type SmallSortFnPtr = fn(&mut [usize], &mut NoOpLogger);
    pub type QuickSelectFnPtr = fn(&mut [usize], usize, &mut NoOpLogger);

    /// Run `sort_fn` on `arr` and verify it produces a sorted permutation.
    /// Emits a `RUNNING: …` stderr line so subprocess TLE diagnostics can
    /// recover the in-flight input.
    fn sort_run_and_verify(
        sort_fn: SortFnPtr,
        name: &str,
        arr: &mut Vec<usize>,
        label: &str,
    ) {
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), "RUNNING: '{}' (n={})", label, arr.len());
        let _ = std::io::stderr().flush();
        let mut expected = arr.clone();
        expected.sort();
        let mut logger = NoOpLogger;
        sort_fn(arr, &mut logger);
        assert_eq!(
            arr, &expected,
            "{}: failed on '{}' (n={})",
            name, label, expected.len()
        );
    }

    /// Generic sort correctness battery. Pattern bank is identical to the
    /// pre-refactor `check_sort` — same inputs, same caps, same assertions
    /// — just with the sort_fn passed in instead of an entry.
    pub fn sort_battery(sort_fn: SortFnPtr, name: &str) {
        let mut rng = thread_rng();
        let cap = max_n_for_tests(name).unwrap_or(usize::MAX);

        macro_rules! check {
            ($arr:expr, $label:expr) => {{
                let arr_vec: Vec<usize> = $arr.into_iter().collect();
                if arr_vec.len() <= cap {
                    sort_run_and_verify(sort_fn, name, &mut { arr_vec }, $label)
                }
            }};
        }
        macro_rules! check_rand {
            ($n:expr, $arr:expr, $label:expr) => {
                check!($arr, $label);
            };
        }

        // ── Trivial cases ────────────────────────────────────────
        check!(vec![], "empty");
        check!(vec![1], "single");
        check!(vec![1, 2], "sorted pair");
        check!(vec![2, 1], "reversed pair");
        check!(vec![1, 1], "equal pair");

        // ── All permutations of small arrays ─────────────────────
        for n in 0..=5 {
            let base: Vec<usize> = (0..n).collect();
            let mut perms: Vec<Vec<usize>> = Vec::new();
            for_each_permutation(&base, &mut |perm| perms.push(perm.to_vec()));
            for perm in perms {
                check!(perm, &format!("perm(n={n})"));
            }
        }

        // ── Structured patterns at several sizes ─────────────────
        for &n in &[16usize, 32, 33, 64, 128] {
            check!(get_arr(n), &format!("sorted {n}"));
            check!(get_reversed_arr(n), &format!("reversed {n}"));
            check!(vec![42usize; n], &format!("all-same {n}"));
        }

        check!(
            (0..128usize).map(|i| if i % 2 == 0 { i } else { 128 - i }).collect::<Vec<_>>(),
            "alternating 128"
        );
        check!(
            (0..100usize).chain((0..99).rev()).collect::<Vec<_>>(),
            "pipe organ 199"
        );
        check!(
            (0..200usize).map(|i| i % 20).collect::<Vec<_>>(),
            "sawtooth 200"
        );

        // Nearly sorted: sorted with a few random swaps
        let mut nearly = get_arr(500);
        for _ in 0..10 {
            let a = rng.gen_range(0..500);
            let b = rng.gen_range(0..500);
            nearly.swap(a, b);
        }
        check_rand!(500, nearly, "nearly sorted 500");

        // Sorted then reversed tail (deterministic, not capped)
        let mut sorted_rev_tail: Vec<usize> = (0..400).collect();
        sorted_rev_tail.extend((400..500).rev());
        check!(sorted_rev_tail, "sorted + reversed tail 500");

        // ── Duplicate-heavy patterns (random within range) ───────
        check_rand!(500, get_rand_arr_in_range(500, 0, 3), "few unique (3 vals, n=500)");
        check_rand!(300, get_rand_arr_in_range(300, 0, 2), "binary (2 vals, n=300)");
        check_rand!(500, get_rand_arr_in_range(500, 0, 50), "many dups (50 vals, n=500)");

        // ── Random cases ─────────────────────────────────────────
        for &n in &[100usize, 500, 1000] {
            let mut perm = get_arr(n);
            perm.shuffle(&mut rng);
            check_rand!(n, perm, &format!("random permutation {n}"));

            check_rand!(
                n,
                get_rand_arr_in_range(n, 0, n),
                &format!("random values {n}")
            );
        }

        check_rand!(5000, get_rand_arr(5000), "random 5000");

        for hook in SORT_PROP_HOOKS {
            hook(sort_fn, name);
        }
    }

    /// Stability battery — runs only on sorts that claim `stable = true`.
    /// Encodes (value, original_index) pairs where comparison is by value
    /// only (value in high bits, index in low bits). After sorting,
    /// equal-valued elements must appear in ascending original-index order.
    pub fn sort_stability_battery(sort_fn: SortFnPtr, name: &str, stable: bool) {
        if !stable {
            return;
        }
        let value_bits = 32;
        let encode = |value: usize, index: usize| -> usize { (value << value_bits) | index };
        let decode_value = |x: usize| -> usize { x >> value_bits };
        let decode_index = |x: usize| -> usize { x & ((1 << value_bits) - 1) };

        let test_cases: Vec<(&str, Vec<usize>)> = vec![
            ("3 values, n=200", (0..200).map(|i| i % 3).collect()),
            ("2 values, n=100", (0..100).map(|i| i % 2).collect()),
            ("all equal, n=50", vec![7; 50]),
            ("10 values, n=500", get_rand_arr_in_range(500, 0, 10)),
        ];

        for (label, values) in &test_cases {
            let mut arr: Vec<usize> = values
                .iter()
                .enumerate()
                .map(|(i, &v)| encode(v, i))
                .collect();
            let mut logger = NoOpLogger;
            sort_fn(&mut arr, &mut logger);

            for i in 1..arr.len() {
                assert!(
                    decode_value(arr[i - 1]) <= decode_value(arr[i]),
                    "{}: stability '{}' — not sorted at position {}",
                    name, label, i
                );
                if decode_value(arr[i - 1]) == decode_value(arr[i]) {
                    assert!(
                        decode_index(arr[i - 1]) < decode_index(arr[i]),
                        "{}: stability '{}' — order violated at position {} \
                         (value={}, indices {} then {})",
                        name, label, i,
                        decode_value(arr[i]),
                        decode_index(arr[i - 1]),
                        decode_index(arr[i]),
                    );
                }
            }
        }
    }

    // ── Component batteries (rotation/partition/merge/small-sort) ────────────
    //
    // These are lighter than the sort battery — each category has a few
    // canonical inputs that exercise its contract. The generated
    // `run_correctness` per family calls into the matching battery.

    /// Verify a rotation produces `arr[split..] ++ arr[..split]` (left
    /// rotation by `split` positions, equivalently: split-point moves to
    /// front).
    pub fn rotation_battery(rotate_fn: RotationFnPtr, name: &str) {
        let cases: Vec<(usize, usize, &str)> = vec![
            (8, 0, "n=8 split=0"),
            (8, 1, "n=8 split=1"),
            (8, 4, "n=8 split=mid"),
            (8, 7, "n=8 split=n-1"),
            (8, 8, "n=8 split=n"),
            (100, 33, "n=100 split=33"),
            (100, 67, "n=100 split=67"),
            (1000, 500, "n=1000 split=mid"),
        ];
        for (n, split, label) in cases {
            let original: Vec<usize> = (0..n).collect();
            let mut arr = original.clone();
            let mut logger = NoOpLogger;
            rotate_fn(&mut arr, split, &mut logger);
            let mut expected: Vec<usize> = original[split..].to_vec();
            expected.extend_from_slice(&original[..split]);
            assert_eq!(
                arr, expected,
                "{}: rotation failed on '{}'",
                name, label
            );
        }

        for hook in ROTATION_PROP_HOOKS {
            hook(rotate_fn, name);
        }
    }

    /// Verify a partition (with pivot selection baked in) produces a
    /// left region of "≤ x" and a right region of "≥ x" for some
    /// boundary value `x` — equivalent to:
    /// `max(arr[..left_end]) ≤ min(arr[right_start..])`. Also checks
    /// the result is a permutation of the input so no element was
    /// dropped or duplicated.
    pub fn partition_battery(partition_fn: PartitionFnPtr, name: &str) {
        // Variety of shapes the cross-product (P × V) has to handle:
        // reverse-sorted (stresses pivot landing), random, already
        // sorted, all-equal, and a small case. Sizes ≥ 16 so that
        // every pivot selector (including MedianOfMedians which wants
        // n ≥ 5 and Ninther which wants n ≥ 9) operates in-band.
        let mut cases: Vec<(Vec<usize>, &str)> = Vec::new();
        cases.push(((0..16).rev().collect(),     "n=16 reverse"));
        cases.push(((0..100).rev().collect(),    "n=100 reverse"));
        cases.push(((0..500).rev().collect(),    "n=500 reverse"));
        cases.push(((0..100).collect(),          "n=100 sorted"));
        cases.push((vec![42usize; 50],           "n=50 all-equal"));
        cases.push((
            (0..200).map(|i| (i * 37 + 13) % 200).collect(),
            "n=200 random-ish",
        ));
        for (mut arr, label) in cases {
            let mut sorted_original = arr.clone();
            sorted_original.sort();
            let mut logger = NoOpLogger;
            let (left_end, right_start) = partition_fn(&mut arr, &mut logger);
            assert!(
                left_end <= right_start,
                "{}: partition '{}' — left_end={} > right_start={}",
                name, label, left_end, right_start
            );
            let max_left = arr[..left_end].iter().copied().max().unwrap_or(usize::MIN);
            let min_right = arr[right_start..].iter().copied().min().unwrap_or(usize::MAX);
            assert!(
                max_left <= min_right,
                "{}: partition '{}' — max_left={} > min_right={} (left_end={}, right_start={})",
                name, label, max_left, min_right, left_end, right_start
            );
            let mut sorted_arr = arr.clone();
            sorted_arr.sort();
            assert_eq!(
                sorted_arr, sorted_original,
                "{}: partition '{}' — output is not a permutation of input",
                name, label
            );
        }

        for hook in PARTITION_PROP_HOOKS {
            hook(partition_fn, name);
        }
    }

    /// Verify a merge produces a sorted array from two pre-sorted halves
    /// (left = `arr[..mid]`, right = `arr[mid..]`).
    pub fn merge_battery(merge_fn: MergeFnPtr, name: &str) {
        let cases: Vec<(usize, usize, &str)> = vec![
            (8, 4, "n=8 mid=4"),
            (9, 4, "n=9 mid=4"),
            (9, 5, "n=9 mid=5"),
            (100, 50, "n=100 mid=50"),
            (500, 250, "n=500 mid=250"),
        ];
        for (n, mid, label) in cases {
            let mut arr: Vec<usize> = Vec::with_capacity(n);
            arr.extend((0..mid).map(|i| 2 * i));
            arr.extend((0..n - mid).map(|i| 2 * i + 1));
            let mut expected = arr.clone();
            expected.sort();
            let mut logger = NoOpLogger;
            merge_fn(&mut arr, mid, &mut logger);
            assert_eq!(arr, expected, "{}: merge failed on '{}'", name, label);
        }

        for hook in MERGE_PROP_HOOKS {
            hook(merge_fn, name);
        }
    }

    /// Verify a quick-select places the target-rank element at
    /// `arr[target]` and leaves the surrounding partitions in the
    /// "all-≤ on the left, all-≥ on the right" shape. Tests several
    /// shapes (reverse, random, all-equal, sorted) and a few target
    /// positions per shape.
    pub fn quick_select_battery(quick_select_fn: QuickSelectFnPtr, name: &str) {
        let shapes: Vec<(Vec<usize>, &str)> = vec![
            ((0..16).rev().collect(),                                     "n=16 reverse"),
            ((0..100).rev().collect(),                                    "n=100 reverse"),
            ((0..200).map(|i| (i * 37 + 13) % 200).collect::<Vec<_>>(),   "n=200 random-ish"),
            ((0..50).chain(0..50).collect::<Vec<_>>(),                    "n=100 dups"),
            (vec![42usize; 50],                                           "n=50 all-equal"),
            ((0..100).collect(),                                          "n=100 sorted"),
        ];
        for (arr_seed, shape_label) in shapes {
            let n = arr_seed.len();
            if n == 0 {
                continue;
            }
            let mut sorted_reference = arr_seed.clone();
            sorted_reference.sort();
            // Pick several target positions per shape — first, last,
            // mid, and a tail-end one — so a one-sided recursion bug
            // surfaces from at least one case.
            let targets: Vec<usize> = if n <= 1 {
                vec![0]
            } else {
                vec![0, n / 2, n - 1, n / 3]
            };
            for target in targets {
                let mut arr = arr_seed.clone();
                let mut logger = NoOpLogger;
                quick_select_fn(&mut arr, target, &mut logger);
                let expected = sorted_reference[target];
                assert_eq!(
                    arr[target], expected,
                    "{}: quick-select '{}' target={} — arr[target]={} expected={}",
                    name, shape_label, target, arr[target], expected
                );
                for (i, &v) in arr[..target].iter().enumerate() {
                    assert!(
                        v <= expected,
                        "{}: quick-select '{}' target={} — arr[{}]={} > pivot={}",
                        name, shape_label, target, i, v, expected
                    );
                }
                for (i, &v) in arr[target + 1..].iter().enumerate() {
                    assert!(
                        v >= expected,
                        "{}: quick-select '{}' target={} — arr[{}]={} < pivot={}",
                        name, shape_label, target, target + 1 + i, v, expected
                    );
                }
                let mut sorted_arr = arr.clone();
                sorted_arr.sort();
                assert_eq!(
                    sorted_arr, sorted_reference,
                    "{}: quick-select '{}' target={} — not a permutation",
                    name, shape_label, target
                );
            }
        }

        for hook in QUICK_SELECT_PROP_HOOKS {
            hook(quick_select_fn, name);
        }
    }

    /// Small-sort battery — only tests sizes within the algorithm's
    /// declared threshold. A small-sort's contract is "len ≤ THRESHOLD";
    /// running it past that is out-of-contract, not a failure of the
    /// algorithm.
    pub fn small_sort_battery(sort_fn: SmallSortFnPtr, name: &str, threshold: usize) {
        for n in 0..=threshold {
            let mut arr: Vec<usize> = (0..n).rev().collect();
            let mut expected = arr.clone();
            expected.sort();
            let mut logger = NoOpLogger;
            sort_fn(&mut arr, &mut logger);
            assert_eq!(arr, expected, "{}: small-sort failed on n={}", name, n);
        }
        // A few random permutations at the upper end.
        if threshold >= 2 {
            let mut rng = thread_rng();
            for trial in 0..10 {
                let mut arr: Vec<usize> = (0..threshold).collect();
                arr.shuffle(&mut rng);
                let mut expected = arr.clone();
                expected.sort();
                let mut logger = NoOpLogger;
                sort_fn(&mut arr, &mut logger);
                assert_eq!(arr, expected, "{}: small-sort random trial {}", name, trial);
            }
        }

        for hook in SMALL_SORT_PROP_HOOKS {
            hook(sort_fn, name, threshold);
        }
    }

    /// Generate all permutations of `base` and call `f` on each (Heap's algorithm).
    fn for_each_permutation(base: &[usize], f: &mut dyn FnMut(&[usize])) {
        let mut arr = base.to_vec();
        let n = arr.len();
        heap_permute(&mut arr, n, f);
    }

    fn heap_permute(arr: &mut Vec<usize>, k: usize, f: &mut dyn FnMut(&[usize])) {
        if k <= 1 {
            f(arr);
            return;
        }
        heap_permute(arr, k - 1, f);
        for i in 0..k - 1 {
            if k % 2 == 0 {
                arr.swap(i, k - 1);
            } else {
                arr.swap(0, k - 1);
            }
            heap_permute(arr, k - 1, f);
        }
    }
}
