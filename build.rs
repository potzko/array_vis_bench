use std::fmt::Write as _;
use std::path::PathBuf;

use combo_codegen::{cross_axis, inline, ComponentDef, Family};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Scan all source files for combo_codegen::component!() annotations.
    let result = combo_codegen::scan("src/").expect("combo_codegen scan failed");
    result.emit_rerun();
    println!("cargo:rerun-if-changed=build.rs");

    let reg = &result.registry;

    // ── Quick sorts ──────────────────────────────────────────────────────────

    let mut qs = String::new();

    // Imports shared by both quick-sort families.
    qs.push_str("use crate::utils::small_sort::{\n");
    qs.push_str("    InsertionSmallSort, Network16SmallSort, NetworkSmallSort, NoSmallSort,\n");
    qs.push_str("};\n");
    qs.push_str("use super::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay};\n");
    qs.push_str("use super::pivot_selectors::{\n");
    qs.push_str(
        "    FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther,\n",
    );
    qs.push_str("    CombinedSelector, NintherDualPivot,\n");
    qs.push_str("};\n");
    qs.push_str("use super::quick_sort::QuickSort;\n");
    qs.push_str("use super::dual_pivot_quick_sort::DualPivotQuickSort;\n\n");

    // Classic single-pivot quicksort: Partition × PivotSelector × SmallSort
    render_sort_family(
        &mut qs,
        Family::new("QuickSort<{P}, {V}, {SS}>")
            .axis("P", reg.role("Partition"))
            .axis("V", reg.role("PivotSelector"))
            .axis("SS", reg.role("SmallSort")),
        FamilyMeta {
            name: "quick sort classic",
            big_o: "O(N Log(N))",
            stable: false,
            direct_sort: true,
            path: &[
                "\"quick sorts\"",
                "\"classic\"",
                "\"{P}\"",
                "\"{V}\"",
                "\"{SS}\"",
            ],
        },
    );

    // Dual-pivot quicksort: DualPivotSelector × SmallSort
    //
    // The DPS axis is the cross-product of all basic pivot selectors (each pair
    // wrapped in CombinedSelector<V1, V2>), plus the hand-written
    // NintherBiasedSelector which targets the ~⅓ and ~⅔ quantiles directly.
    let pivots = reg.role("PivotSelector");
    let mut dual_pivot_axis = cross_axis(
        pivots,
        pivots,
        |v1, v2| format!("CombinedSelector<{}, {}>", v1.type_expr, v2.type_expr),
        |v1, v2| format!("{} / {}", v1.label, v2.label),
    );
    dual_pivot_axis.push(ComponentDef::new("NintherDualPivot", "ninther 1/3 + 2/3"));

    render_sort_family(
        &mut qs,
        Family::new("DualPivotQuickSort<{DPS}, {SS}>")
            .axis("DPS", &dual_pivot_axis)
            .axis("SS", reg.role("SmallSort")),
        FamilyMeta {
            name: "quick sort dual pivot",
            big_o: "O(N Log(N))",
            stable: false,
            direct_sort: true,
            path: &[
                "\"quick sorts\"",
                "\"dual pivot\"",
                "\"{DPS}\"",
                "\"{SS}\"",
            ],
        },
    );

    std::fs::write(out_dir.join("quick_sorts_combinations.rs"), qs)
        .expect("failed to write quick_sorts_combinations.rs");

    // ── Merge sorts ──────────────────────────────────────────────────────────

    let mut ms = String::new();

    ms.push_str("use crate::sorts::merge_sorts::top_down::TopDownMergeSort;\n");
    ms.push_str("use crate::sorts::merge_sorts::bottom_up::BottomUpMergeSort;\n");
    ms.push_str(
        "use crate::sorts::merge_sorts::top_down_mirror::TopDownMirrorMergeSort;\n",
    );
    ms.push_str("use crate::sorts::merge_sorts::naive::NaiveMergeSort;\n");
    ms.push_str("use crate::sorts::merge_sorts::natural::NaturalMergeSort;\n");
    ms.push_str("use crate::sorts::merge_sorts::timsort::TimSort;\n");
    ms.push_str("use crate::utils::small_sort::{\n");
    ms.push_str(
        "    NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort,\n",
    );
    ms.push_str("};\n");
    ms.push_str(
        "use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;\n",
    );
    ms.push_str(
        "use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;\n",
    );
    ms.push_str(
        "use crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};\n",
    );
    ms.push_str("use crate::utils::rotation::{\n");
    ms.push_str("    ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation,\n");
    ms.push_str("    TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation,\n");
    ms.push_str("    HelixRotation, DrillRotation, JugglingRotation,\n");
    ms.push_str("};\n\n");

    let ss = reg.role("SmallSort");
    let rot = reg.role("Rotation");
    let pp = inline(&[("false", ""), ("true", "ping-pong")]);
    let ee = inline(&[("false", ""), ("true", "early-exit")]);
    let gallop = inline(&[("false", ""), ("true", "gallop")]);

    // Classic merge sorts
    render_sort_family(
        &mut ms,
        Family::new("TopDownMergeSort<{SS}, {PP}, {EE}>")
            .axis("SS", ss)
            .axis("PP", &pp)
            .axis("EE", &ee),
        FamilyMeta {
            name: "merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &["\"merge sorts\"", "\"classic\"", "\"top-down\"", "\"{variant}\""],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("BottomUpMergeSort<{SS}, {PP}, {EE}>")
            .axis("SS", ss)
            .axis("PP", &pp)
            .axis("EE", &ee),
        FamilyMeta {
            name: "bottom-up merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &["\"merge sorts\"", "\"classic\"", "\"bottom-up\"", "\"{variant}\""],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("TopDownMirrorMergeSort<{SS}, {PP}, {EE}>")
            .axis("SS", ss)
            .axis("PP", &pp)
            .axis("EE", &ee),
        FamilyMeta {
            name: "top-down mirror merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"classic\"",
                "\"top-down mirror\"",
                "\"{variant}\"",
            ],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("NaiveMergeSort<{SS}>").axis("SS", ss),
        FamilyMeta {
            name: "naive merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &["\"merge sorts\"", "\"classic\"", "\"naive\"", "\"{variant}\""],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("NaturalMergeSort<{PP}, {EE}>")
            .axis("PP", &pp)
            .axis("EE", &ee),
        FamilyMeta {
            name: "natural merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &["\"merge sorts\"", "\"classic\"", "\"natural\"", "\"{variant}\""],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("TimSort<{Gallop}>").axis("Gallop", &gallop),
        FamilyMeta {
            name: "timsort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"miscellaneous\"",
                "\"timsort\"",
                "\"{variant}\"",
            ],
        },
    );

    // Rotation merge sorts: {top-down, bottom-up} × {naive, smaller-side} merge
    render_sort_family(
        &mut ms,
        Family::new("TopDownRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>")
            .axis("R", rot)
            .axis("SS", ss),
        FamilyMeta {
            name: "rotation merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"rotation\"",
                "\"top-down\"",
                "\"{R}\"",
                "\"{SS}\"",
            ],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("TopDownRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>")
            .axis("R", rot)
            .axis("SS", ss),
        FamilyMeta {
            name: "rotation merge sort<smaller-side>",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"rotation\"",
                "\"top-down smaller-side\"",
                "\"{R}\"",
                "\"{SS}\"",
            ],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("BottomUpRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>")
            .axis("R", rot)
            .axis("SS", ss),
        FamilyMeta {
            name: "bottom-up rotation merge sort",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"rotation\"",
                "\"bottom-up\"",
                "\"{R}\"",
                "\"{SS}\"",
            ],
        },
    );

    render_sort_family(
        &mut ms,
        Family::new("BottomUpRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>")
            .axis("R", rot)
            .axis("SS", ss),
        FamilyMeta {
            name: "bottom-up rotation merge sort<smaller-side>",
            big_o: "O(N log N)",
            stable: true,
            direct_sort: true,
            path: &[
                "\"merge sorts\"",
                "\"rotation\"",
                "\"bottom-up smaller-side\"",
                "\"{R}\"",
                "\"{SS}\"",
            ],
        },
    );

    std::fs::write(out_dir.join("merge_sorts_combinations.rs"), ms)
        .expect("failed to write merge_sorts_combinations.rs");
}

// ── Code generation helpers ──────────────────────────────────────────────────

struct FamilyMeta<'a> {
    name: &'a str,
    big_o: &'a str,
    stable: bool,
    direct_sort: bool,
    /// Each element is either a quoted literal like `"\"quick sorts\""` or a
    /// `"{VAR}"` placeholder that will be used verbatim in the generated macro.
    path: &'a [&'a str],
}

/// Emit one `sort_registry_macro::sort_family! { … }` invocation into `out`.
fn render_sort_family(out: &mut String, family: Family, meta: FamilyMeta<'_>) {
    out.push_str("sort_registry_macro::sort_family! {\n");
    writeln!(out, "    type Sort = {};", family.type_template).unwrap();
    out.push('\n');

    for axis in family.axes() {
        writeln!(out, "    {} {{", axis.var).unwrap();
        for comp in &axis.components {
            writeln!(out, "        {} => \"{}\"", comp.type_expr, comp.label).unwrap();
        }
        out.push_str("    }\n");
    }

    out.push('\n');
    writeln!(out, "    name        = \"{}\";", meta.name).unwrap();
    writeln!(out, "    big_o       = \"{}\";", meta.big_o).unwrap();
    writeln!(out, "    stable      = {};", meta.stable).unwrap();
    writeln!(out, "    direct_sort = {};", meta.direct_sort).unwrap();

    let path_str = meta.path.join(", ");
    writeln!(out, "    path        = [{path_str}];").unwrap();

    out.push_str("}\n\n");
}
