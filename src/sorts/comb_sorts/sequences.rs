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
// Shrink-factor sequences
//
// Each variant is defined by a rational approximation of a shrink factor.
// The gaps for an array of length n are:  n, n/k, n/k², ..., 1
// where k is the shrink factor (as a float).
//
// Usage:  register_comb!(mod_name, DISPLAY_NAME, NUMERATOR, DENOMINATOR)
// NUMERATOR/DENOMINATOR is the rational approximation of the shrink factor.
// ---------------------------------------------------------------------------
macro_rules! register_comb {
    ($mod:ident, $display:literal, $num:literal, $den:literal) => {
        mod $mod {
            use crate::sorts::comb_sorts::comb_sort::CombSort;
            use crate::sorts::comb_sorts::sequences::{CombEntry, COMB_SEQUENCES};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const DISPLAY: &str = $display;
            const SORT_NAME: &str =
                const_format::concatcp!("comb sort (shrink ", $display, ")");
            const PATH: &[&str] = &["comb sorts", DISPLAY];

            fn gaps(n: usize) -> Vec<usize> {
                let mut g = n;
                let mut gs = Vec::new();
                while g > 1 {
                    g = (g * $den / $num).max(1);
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
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                CombSort::sort_with_gaps(arr, &mut l, gaps(arr.len()));
            }

            #[linkme::distributed_slice(COMB_SEQUENCES)]
            static ENTRY: CombEntry = CombEntry {
                name: SORT_NAME,
                big_o: "O(N^2)",
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry {
                    name: SORT_NAME,
                    big_o: "O(N^2)",
                    stable: false,
                    run: bench,
                };
        }
    };
}

// Classic (1.3 = 13/10) — the original Dobosiewicz/Box shrink factor
register_comb!(classic,      "1.3",          10, 13);
// sqrt(2) ≈ 99/70
register_comb!(sqrt2,        "√2 ≈ 1.414",   70, 99);
// Golden ratio φ ≈ 89/55
register_comb!(phi,          "φ ≈ 1.618",    55, 89);
// 4/3 — simple rational close to 1.3, faster gap decay
register_comb!(four_thirds,  "4/3",           3,  4);
// 11/8 — suggested in Lacey & Box (1991)
register_comb!(eleven_eighths, "11/8",        8, 11);
// 5/4 — faster convergence, coarser early passes
register_comb!(five_fourths, "5/4",           4,  5);
