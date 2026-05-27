//! Build-time scanner and cross-product generator for annotated type components.
//!
//! # Usage overview
//!
//! ## 1. Declare components in Cargo.toml metadata
//!
//! Each component is one entry in the
//! `[package.metadata.array_vis_bench.components]` array-of-tables:
//!
//! ```toml
//! [[package.metadata.array_vis_bench.components]]
//! role  = "Partition"
//! type  = "LeftLeftPartition"
//! label = "left-left pointer"
//!
//! [[package.metadata.array_vis_bench.components]]
//! role  = "SmallSort"
//! type  = "InsertionSmallSort<LinearInsertion, 16>"
//! label = "insertion: 16"
//! ```
//!
//! Unknown fields error at scan time (`deny_unknown_fields`); a typo'd
//! key surfaces as a build failure pointing at the manifest. See
//! [`metadata_scanner::scan_manifest`].
//!
//! ## 2. Annotate families next to their struct definitions
//!
//! ```rust,ignore
//! combo_codegen::family!(
//!     type = QuickSort<{P}, {V}, {SS}>,
//!     uses = [
//!         "super::partitions::{LeftLeftPartition, LeftRightPartition}",
//!         "super::pivot_selectors::FirstElement",
//!         "super::quick_sort::QuickSort",
//!     ],
//!     P: Partition,
//!     V: PivotSelector,
//!     SS: SmallSort,
//!     name = "quick sort",
//!     big_o = "O(N log N)",
//!     stable = false,
//!     direct_sort = true,
//!     path = ["quick sorts", "{P}", "{V}", "{SS}"],
//! );
//! ```
//!
//! Both macros **expand to nothing** — they are purely markers for the build
//! scanner. The legacy [`sort_family!`] name is also accepted (the
//! `CodegenConfig::for_sort_families` preset selects it as the scanner
//! marker).
//!
//! ## 3. Scan and generate in `build.rs`
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
//!     let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
//!     let config = combo_codegen::CodegenConfig::for_sort_families();
//!     let mut result = combo_codegen::scan("src/", &config).unwrap();
//!     for c in combo_codegen::scan_manifest(manifest_dir.join("Cargo.toml")).unwrap().iter().rev() {
//!         result.registry.add_front(c.role.clone(), c.type_expr.clone(), c.label.clone());
//!     }
//!     result.validate().unwrap();
//!     result.emit_rerun();
//!     println!("cargo:rerun-if-changed=build.rs");
//!     result.emit_families(&out_dir).unwrap();
//! }
//! ```
//!
//! For a non-sort use case, build your own [`CodegenConfig`]:
//!
//! ```rust,ignore
//! let config = combo_codegen::CodegenConfig::new("my_family", "my_crate::my_macro")
//!     .with_type_prefix("type Target = ")
//!     .with_path_field("menu");
//! ```
//!
//! ## 4. Consume the generated file
//!
//! ```rust,ignore
//! pub mod combinations {
//!     include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
//! }
//! ```

pub mod family;
pub mod metadata_scanner;
pub mod scanner;

pub use family::{
    cross_axis, expand_role, inline, Axis, AxisSpec, CodegenConfig, Combination, ComponentDef,
    ComponentRegistry, Family, FamilyDef, FieldValue, Slot,
};
pub use metadata_scanner::{
    scan_manifest, scan_workspace_components, scan_workspace_families, MetadataComponent,
    MetadataError, MetadataFamily,
};
pub use scanner::{scan, ScanResult, ValidationError};

// ── family! / sort_family! markers ───────────────────────────────────────────

/// Declare a family inline, next to its struct definition.
///
/// This macro **expands to nothing**. It exists solely as a marker that the
/// build-script scanner ([`scan`]) recognises and uses to generate
/// `<module><config.filename_suffix>` files via
/// [`ScanResult::emit_families`].
#[macro_export]
macro_rules! family {
    ($($tt:tt)*) => {};
}
