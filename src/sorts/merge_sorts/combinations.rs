use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;

pub struct MergeSortEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    pub path: &'static [&'static str],
    pub sort_fn: SortFn,
    pub sort_vis: fn(&mut [usize], &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static MERGE_SORTS: [MergeSortEntry] = [..];

#[ctor::ctor]
fn register_merge_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();
    for entry in MERGE_SORTS {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, true, entry.path);
    }
}

// ---------------------------------------------------------------------------
// register_merge_sort!(mod_name, SortType, "name", "big_o", path_expr)
// ---------------------------------------------------------------------------
macro_rules! register_merge_sort {
    ($mod:ident, $sort_ty:ty, $name:literal, $big_o:literal, $path:expr) => {
        mod $mod {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::top_down::TopDownMergeSort;
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::bottom_up::BottomUpMergeSort;
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::natural::NaturalMergeSort;
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::top_down_mirror::TopDownMirrorMergeSort;
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::rotation::{TopDownRotationMergeSort, BottomUpRotationMergeSort};
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};
            #[allow(unused_imports)]
            use crate::utils::rotation::{
                ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation,
                TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation,
                HelixRotation, DrillRotation, JugglingRotation,
            };
            #[allow(unused_imports)]
            use crate::sorts::merge_sorts::small_sort::{NoSmallSort, InsertionSmallSort};

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                <$sort_ty>::sort(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                <$sort_ty>::sort(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                <$sort_ty>::sort(arr, &mut l);
            }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry {
                name: $name,
                big_o: $big_o,
                path: $path,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry {
                    name: $name,
                    big_o: $big_o,
                    stable: true,
                    run: bench,
                };
        }
    };
}


// ---------------------------------------------------------------------------
// Top-down  (THRESHOLD, PING_PONG, EARLY_EXIT)
// ---------------------------------------------------------------------------
register_merge_sort!(td_classic,           TopDownMergeSort::<NoSmallSort,          false, false>, "merge sort",                                       "O(N log N)", &["merge sorts", "classic", "top-down", "classic"]);
register_merge_sort!(td_pp,                TopDownMergeSort::<NoSmallSort,          true,  false>, "merge sort<ping-pong>",                            "O(N log N)", &["merge sorts", "classic", "top-down", "ping-pong"]);
register_merge_sort!(td_ee,                TopDownMergeSort::<NoSmallSort,          false, true>,  "merge sort<early-exit>",                           "O(N log N)", &["merge sorts", "classic", "top-down", "early-exit"]);
register_merge_sort!(td_pp_ee,             TopDownMergeSort::<NoSmallSort,          true,  true>,  "merge sort<ping-pong, early-exit>",                "O(N log N)", &["merge sorts", "classic", "top-down", "ping-pong + early-exit"]);
register_merge_sort!(td_t32,               TopDownMergeSort::<InsertionSmallSort<32>, false, false>, "merge sort<threshold: 32>",                     "O(N log N)", &["merge sorts", "classic", "top-down", "threshold 32"]);
register_merge_sort!(td_t32_pp,            TopDownMergeSort::<InsertionSmallSort<32>, true,  false>, "merge sort<threshold: 32, ping-pong>",          "O(N log N)", &["merge sorts", "classic", "top-down", "threshold 32 + ping-pong"]);
register_merge_sort!(td_t32_ee,            TopDownMergeSort::<InsertionSmallSort<32>, false, true>,  "merge sort<threshold: 32, early-exit>",         "O(N log N)", &["merge sorts", "classic", "top-down", "threshold 32 + early-exit"]);
register_merge_sort!(td_t32_pp_ee,         TopDownMergeSort::<InsertionSmallSort<32>, true,  true>,  "merge sort<threshold: 32, ping-pong, early-exit>","O(N log N)", &["merge sorts", "classic", "top-down", "threshold 32 + ping-pong + early-exit"]);

// ---------------------------------------------------------------------------
// Bottom-up  (THRESHOLD, PING_PONG, EARLY_EXIT)
// ---------------------------------------------------------------------------
register_merge_sort!(bu_pow2,              BottomUpMergeSort::<NoSmallSort,          false, false>, "bottom-up merge sort",                             "O(N log N)", &["merge sorts", "classic", "bottom-up", "classic"]);
register_merge_sort!(bu_pow2_pp,           BottomUpMergeSort::<NoSmallSort,          true,  false>, "bottom-up merge sort<ping-pong>",                  "O(N log N)", &["merge sorts", "classic", "bottom-up", "ping-pong"]);
register_merge_sort!(bu_pow2_ee,           BottomUpMergeSort::<NoSmallSort,          false, true>,  "bottom-up merge sort<early-exit>",                 "O(N log N)", &["merge sorts", "classic", "bottom-up", "early-exit"]);
register_merge_sort!(bu_pow2_pp_ee,        BottomUpMergeSort::<NoSmallSort,          true,  true>,  "bottom-up merge sort<ping-pong, early-exit>",      "O(N log N)", &["merge sorts", "classic", "bottom-up", "ping-pong + early-exit"]);
register_merge_sort!(bu_t32,               BottomUpMergeSort::<InsertionSmallSort<32>, false, false>, "bottom-up merge sort<threshold: 32>",           "O(N log N)", &["merge sorts", "classic", "bottom-up", "threshold 32"]);
register_merge_sort!(bu_t32_pp,            BottomUpMergeSort::<InsertionSmallSort<32>, true,  false>, "bottom-up merge sort<threshold: 32, ping-pong>","O(N log N)", &["merge sorts", "classic", "bottom-up", "threshold 32 + ping-pong"]);
register_merge_sort!(bu_t32_ee,            BottomUpMergeSort::<InsertionSmallSort<32>, false, true>,  "bottom-up merge sort<threshold: 32, early-exit>","O(N log N)", &["merge sorts", "classic", "bottom-up", "threshold 32 + early-exit"]);
register_merge_sort!(bu_t32_pp_ee,         BottomUpMergeSort::<InsertionSmallSort<32>, true,  true>,  "bottom-up merge sort<threshold: 32, ping-pong, early-exit>","O(N log N)", &["merge sorts", "classic", "bottom-up", "threshold 32 + ping-pong + early-exit"]);

// ---------------------------------------------------------------------------
// Top-down mirror  (SmallSort, PING_PONG, EARLY_EXIT)
// ---------------------------------------------------------------------------
register_merge_sort!(tdm,                  TopDownMirrorMergeSort::<NoSmallSort,          false, false>, "top-down mirror merge sort",                        "O(N log N)", &["merge sorts", "classic", "top-down mirror", "classic"]);
register_merge_sort!(tdm_pp,               TopDownMirrorMergeSort::<NoSmallSort,          true,  false>, "top-down mirror merge sort<ping-pong>",             "O(N log N)", &["merge sorts", "classic", "top-down mirror", "ping-pong"]);
register_merge_sort!(tdm_ee,               TopDownMirrorMergeSort::<NoSmallSort,          false, true>,  "top-down mirror merge sort<early-exit>",            "O(N log N)", &["merge sorts", "classic", "top-down mirror", "early-exit"]);
register_merge_sort!(tdm_pp_ee,            TopDownMirrorMergeSort::<NoSmallSort,          true,  true>,  "top-down mirror merge sort<ping-pong, early-exit>", "O(N log N)", &["merge sorts", "classic", "top-down mirror", "ping-pong + early-exit"]);
register_merge_sort!(tdm_t32,              TopDownMirrorMergeSort::<InsertionSmallSort<32>, false, false>, "top-down mirror merge sort<threshold: 32>",       "O(N log N)", &["merge sorts", "classic", "top-down mirror", "threshold 32"]);
register_merge_sort!(tdm_t32_pp,           TopDownMirrorMergeSort::<InsertionSmallSort<32>, true,  false>, "top-down mirror merge sort<threshold: 32, ping-pong>","O(N log N)", &["merge sorts", "classic", "top-down mirror", "threshold 32 + ping-pong"]);
register_merge_sort!(tdm_t32_ee,           TopDownMirrorMergeSort::<InsertionSmallSort<32>, false, true>,  "top-down mirror merge sort<threshold: 32, early-exit>","O(N log N)", &["merge sorts", "classic", "top-down mirror", "threshold 32 + early-exit"]);
register_merge_sort!(tdm_t32_pp_ee,        TopDownMirrorMergeSort::<InsertionSmallSort<32>, true,  true>,  "top-down mirror merge sort<threshold: 32, ping-pong, early-exit>","O(N log N)", &["merge sorts", "classic", "top-down mirror", "threshold 32 + ping-pong + early-exit"]);

// ---------------------------------------------------------------------------
// register_rotation!(mod_td, mod_td_ss, mod_bu, mod_bu_ss,
//                    mod_td_t, mod_td_ss_t, mod_bu_t, mod_bu_ss_t, RotationType)
//
// Generates 8 rotation merge sort variants for a given rotation algorithm:
//   4 without small-sort + 4 with InsertionSmallSort<32> threshold.
// To add a new rotation: add NAME to its Rotation impl, then add one line here.
// ---------------------------------------------------------------------------
macro_rules! register_rotation {
    ($mod_td:ident, $mod_td_ss:ident, $mod_bu:ident, $mod_bu_ss:ident,
     $mod_td_t:ident, $mod_td_ss_t:ident, $mod_bu_t:ident, $mod_bu_ss_t:ident,
     $rot:ident) => {
        mod $mod_td {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::NaiveRotationMerge;
            use crate::sorts::merge_sorts::small_sort::NoSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("rotation merge sort<", $rot::NAME, ">");
            const PATH: &[&str] = &["merge sorts", "rotation", "top-down", $rot::NAME];
            type S = TopDownRotationMergeSort<NoSmallSort, NaiveRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_td_ss {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::SmallerSideRotationMerge;
            use crate::sorts::merge_sorts::small_sort::NoSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("rotation merge sort<", $rot::NAME, ", smaller-side>");
            const PATH: &[&str] = &["merge sorts", "rotation", "top-down smaller-side", $rot::NAME];
            type S = TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_bu {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::NaiveRotationMerge;
            use crate::sorts::merge_sorts::small_sort::NoSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("bottom-up rotation merge sort<", $rot::NAME, ">");
            const PATH: &[&str] = &["merge sorts", "rotation", "bottom-up", $rot::NAME];
            type S = BottomUpRotationMergeSort<NoSmallSort, NaiveRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_bu_ss {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::SmallerSideRotationMerge;
            use crate::sorts::merge_sorts::small_sort::NoSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("bottom-up rotation merge sort<", $rot::NAME, ", smaller-side>");
            const PATH: &[&str] = &["merge sorts", "rotation", "bottom-up smaller-side", $rot::NAME];
            type S = BottomUpRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        // --- threshold-32 variants ---
        mod $mod_td_t {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::NaiveRotationMerge;
            use crate::sorts::merge_sorts::small_sort::InsertionSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("rotation merge sort<", $rot::NAME, ", threshold: 32>");
            const PATH: &[&str] = &["merge sorts", "rotation", "top-down, threshold 32", $rot::NAME];
            type S = TopDownRotationMergeSort<InsertionSmallSort<32>, NaiveRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_td_ss_t {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::SmallerSideRotationMerge;
            use crate::sorts::merge_sorts::small_sort::InsertionSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("rotation merge sort<", $rot::NAME, ", smaller-side, threshold: 32>");
            const PATH: &[&str] = &["merge sorts", "rotation", "top-down smaller-side, threshold 32", $rot::NAME];
            type S = TopDownRotationMergeSort<InsertionSmallSort<32>, SmallerSideRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_bu_t {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::NaiveRotationMerge;
            use crate::sorts::merge_sorts::small_sort::InsertionSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("bottom-up rotation merge sort<", $rot::NAME, ", threshold: 32>");
            const PATH: &[&str] = &["merge sorts", "rotation", "bottom-up, threshold 32", $rot::NAME];
            type S = BottomUpRotationMergeSort<InsertionSmallSort<32>, NaiveRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
        mod $mod_bu_ss_t {
            use super::{MergeSortEntry, MERGE_SORTS};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;
            use crate::sorts::merge_sorts::rotation_merge::SmallerSideRotationMerge;
            use crate::sorts::merge_sorts::small_sort::InsertionSmallSort;
            use crate::utils::rotation::{Rotation, $rot};

            const NAME: &str = const_format::concatcp!("bottom-up rotation merge sort<", $rot::NAME, ", smaller-side, threshold: 32>");
            const PATH: &[&str] = &["merge sorts", "rotation", "bottom-up smaller-side, threshold 32", $rot::NAME];
            type S = BottomUpRotationMergeSort<InsertionSmallSort<32>, SmallerSideRotationMerge<$rot>, false>;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { S::sort(arr, logger); }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { S::sort(arr, logger); }
            fn bench(arr: &mut [usize]) { let mut l = NoOpLogger; S::sort(arr, &mut l); }

            #[linkme::distributed_slice(MERGE_SORTS)]
            static ENTRY: MergeSortEntry = MergeSortEntry { name: NAME, big_o: "O(N log N)", path: PATH, sort_fn, sort_vis };
            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry { name: NAME, big_o: "O(N log N)", stable: true, run: bench };
        }
    };
}

// To add a new rotation algorithm:
//   1. Add the struct + Rotation impl (with NAME) to src/utils/rotation/<name>.rs
//   2. Re-export it from src/utils/rotation/mod.rs
//   3. Add one register_rotation! line here — nothing else changes
register_rotation!(rot_td_rev, rot_td_ss_rev, rot_bu_rev, rot_bu_ss_rev, rot_td_t_rev, rot_td_ss_t_rev, rot_bu_t_rev, rot_bu_ss_t_rev, ReversalRotation);
register_rotation!(rot_td_aux, rot_td_ss_aux, rot_bu_aux, rot_bu_ss_aux, rot_td_t_aux, rot_td_ss_t_aux, rot_bu_t_aux, rot_bu_ss_t_aux, AuxiliaryRotation);
register_rotation!(rot_td_bri, rot_td_ss_bri, rot_bu_bri, rot_bu_ss_bri, rot_td_t_bri, rot_td_ss_t_bri, rot_bu_t_bri, rot_bu_ss_t_bri, BridgeRotation);
register_rotation!(rot_td_ctr, rot_td_ss_ctr, rot_bu_ctr, rot_bu_ss_ctr, rot_td_t_ctr, rot_td_ss_t_ctr, rot_bu_t_ctr, rot_bu_ss_t_ctr, ContrevRotation);
register_rotation!(rot_td_tri, rot_td_ss_tri, rot_bu_tri, rot_bu_ss_tri, rot_td_t_tri, rot_td_ss_t_tri, rot_bu_t_tri, rot_bu_ss_t_tri, TrinityRotation);
register_rotation!(rot_td_gm,  rot_td_ss_gm,  rot_bu_gm,  rot_bu_ss_gm,  rot_td_t_gm,  rot_td_ss_t_gm,  rot_bu_t_gm,  rot_bu_ss_t_gm,  GriesMillsRotation);
register_rotation!(rot_td_gra, rot_td_ss_gra, rot_bu_gra, rot_bu_ss_gra, rot_td_t_gra, rot_td_ss_t_gra, rot_bu_t_gra, rot_bu_ss_t_gra, GrailRotation);
register_rotation!(rot_td_pis, rot_td_ss_pis, rot_bu_pis, rot_bu_ss_pis, rot_td_t_pis, rot_td_ss_t_pis, rot_bu_t_pis, rot_bu_ss_t_pis, PistonRotation);
register_rotation!(rot_td_hel, rot_td_ss_hel, rot_bu_hel, rot_bu_ss_hel, rot_td_t_hel, rot_td_ss_t_hel, rot_bu_t_hel, rot_bu_ss_t_hel, HelixRotation);
register_rotation!(rot_td_dri, rot_td_ss_dri, rot_bu_dri, rot_bu_ss_dri, rot_td_t_dri, rot_td_ss_t_dri, rot_bu_t_dri, rot_bu_ss_t_dri, DrillRotation);
register_rotation!(rot_td_jug, rot_td_ss_jug, rot_bu_jug, rot_bu_ss_jug, rot_td_t_jug, rot_td_ss_t_jug, rot_bu_t_jug, rot_bu_ss_t_jug, JugglingRotation);

// ---------------------------------------------------------------------------
// Natural  (PING_PONG, EARLY_EXIT)
// ---------------------------------------------------------------------------
register_merge_sort!(nat,                  NaturalMergeSort::<false, false>, "natural merge sort",                                "O(N log N)", &["merge sorts", "classic", "natural", "classic"]);
register_merge_sort!(nat_pp,               NaturalMergeSort::<true,  false>, "natural merge sort<ping-pong>",                    "O(N log N)", &["merge sorts", "classic", "natural", "ping-pong"]);
register_merge_sort!(nat_ee,               NaturalMergeSort::<false, true>,  "natural merge sort<early-exit>",                   "O(N log N)", &["merge sorts", "classic", "natural", "early-exit"]);
register_merge_sort!(nat_pp_ee,            NaturalMergeSort::<true,  true>,  "natural merge sort<ping-pong, early-exit>",        "O(N log N)", &["merge sorts", "classic", "natural", "ping-pong + early-exit"]);
