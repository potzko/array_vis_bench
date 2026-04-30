use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;

pub struct CombEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    /// Navigation path for the tree menu, e.g. `["comb sorts", "1.3"]`.
    pub path: &'static [&'static str],
    pub sort_fn: SortFn,
    pub sort_vis: fn(&mut [usize], &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static COMB_SEQUENCES: [CombEntry] = [..];

// ---------------------------------------------------------------------------
// Visualization-only registration (COMB_SEQUENCES for fn_sort dispatch).
//
// Each variant is registered in COMB_SEQUENCES so that the interactive
// visualiser can find it by name.  BENCH_SORTS + SORT_REGISTRY are handled
// separately by the `sort_family!` call below.
// ---------------------------------------------------------------------------
macro_rules! register_comb_vis {
    ($mod:ident, $display:literal, $num:literal, $den:literal) => {
        mod $mod {
            use crate::sorts::comb_sorts::comb_sort::CombSort;
            use crate::sorts::comb_sorts::sequences::{CombEntry, COMB_SEQUENCES};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str =
                const_format::concatcp!("comb sort (shrink ", $display, ")");
            const PATH: &[&str] = &["comb sorts", $display];

            fn gaps(n: usize) -> Vec<usize> {
                let mut g = n;
                let mut gs = Vec::new();
                while g > 1 {
                    g = (g * $num / $den).max(1);
                    gs.push(g);
                }
                if gs.last() != Some(&1) {
                    gs.push(1);
                }
                gs
            }

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                CombSort::sort_with_gaps(arr, logger, gaps(arr.len()));
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                CombSort::sort_with_gaps(arr, logger, gaps(arr.len()));
            }

            #[linkme::distributed_slice(COMB_SEQUENCES)]
            static ENTRY: CombEntry = CombEntry {
                name: SORT_NAME,
                big_o: "O(N^2)",
                path: PATH,
                sort_fn,
                sort_vis,
            };
        }
    };
}

// Classic (1.3 = 13/10) — the original Dobosiewicz/Box shrink factor
register_comb_vis!(classic,           "1.3",        10, 13);
// sqrt(2) ≈ 99/70
register_comb_vis!(sqrt2,             "√2 ≈ 1.414", 70, 99);
// Golden ratio φ ≈ 89/55
register_comb_vis!(phi,               "φ ≈ 1.618",  55, 89);
// 4/3 — simple rational close to 1.3, faster gap decay
register_comb_vis!(four_thirds,       "4/3",          3,  4);
// 11/8 — suggested in Lacey & Box (1991)
register_comb_vis!(eleven_eighths,    "11/8",          8, 11);
// 5/4 — faster convergence, coarser early passes
register_comb_vis!(five_fourths,      "5/4",           4,  5);

// ---------------------------------------------------------------------------
// Bench + sort-registry via sort_family!
//
// CombSortRatio<NUM, DEN> implements SortAlgo and computes the same gaps as
// the visualisation closures above, so benchmark results are consistent.
// The sort name format matches the COMB_SEQUENCES names exactly so that the
// interactive UI and benchmarks refer to the same sort.
// ---------------------------------------------------------------------------
combo_codegen::sort_family!(
    type = {R},
    uses = [
        "crate::sorts::comb_sorts::comb_sort_ratio::CombSortRatio",
    ],
    R: CombRatio,
    name = "comb sort",
    big_o = "O(N^2)",
    stable = false,
    direct_sort = false,
    path = ["comb sorts", "{R}"],
);
