use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;
use crate::sorts::merge_sorts::top_down::TopDownMergeSort;
use crate::sorts::merge_sorts::bottom_up::BottomUpMergeSort;
use crate::sorts::merge_sorts::top_down_mirror::TopDownMirrorMergeSort;
use crate::sorts::merge_sorts::naive::NaiveMergeSort;
use crate::sorts::merge_sorts::natural::NaturalMergeSort;
use crate::sorts::merge_sorts::timsort::TimSort;
use crate::sorts::merge_sorts::small_sort::{NoSmallSort, InsertionSmallSort};

// ---------------------------------------------------------------------------
// MERGE_SORTS distributed slice — used only for rotation variants that still
// use register_rotation!.  All other sorts register directly via sort_family!
// into SORT_REGISTRY and SORT_VIS_REGISTRY.
// ---------------------------------------------------------------------------

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
fn register_rotation_sorts() {
    let mut registry = crate::traits::SORT_REGISTRY.lock().unwrap();
    let mut vis_registry = crate::traits::SORT_VIS_REGISTRY.lock().unwrap();
    for entry in MERGE_SORTS {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        vis_registry.insert(entry.name.to_string(), entry.sort_vis);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, true, entry.path);
    }
}

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
// Non-rotation sorts via sort_family!
//
// Each generates 2×2×2 = 8 concrete monomorphisations (or 2×2 / 2 for sorts
// with fewer parameters).  All names match what register_merge_sort! produced
// previously, so existing tests and speed_test output are unchanged.
// ---------------------------------------------------------------------------

sort_registry_macro::sort_family! {
    type Sort = TopDownMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => ""
        InsertionSmallSort<32> => "threshold: 32"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "top-down", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = BottomUpMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => ""
        InsertionSmallSort<32> => "threshold: 32"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "bottom-up merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "bottom-up", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = TopDownMirrorMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => ""
        InsertionSmallSort<32> => "threshold: 32"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "top-down mirror merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "top-down mirror", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = NaiveMergeSort<{SS}>;

    SS {
        NoSmallSort            => ""
        InsertionSmallSort<32> => "threshold: 32"
    }

    name        = "naive merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "naive", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = NaturalMergeSort<{PP}, {EE}>;

    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "natural merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "natural", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = TimSort<{Gallop}>;

    Gallop {
        false => ""
        true  => "gallop"
    }

    name        = "timsort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "miscellaneous", "timsort", "{variant}"];
}
