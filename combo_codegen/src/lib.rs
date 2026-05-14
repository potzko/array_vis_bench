//! Build-time scanner and cross-product generator for annotated type components.
//!
//! # Usage overview
//!
//! ## 1. Annotate concrete types in source files
//!
//! Place `component!` calls next to your type definitions:
//!
//! ```rust,ignore
//! // partitions.rs
//! combo_codegen::component!(Partition, Lomuto, "lomuto");
//! combo_codegen::component!(Partition, Hoare, "hoare");
//!
//! // Works with const-generic types too:
//! combo_codegen::component!(SmallSort, InsertionSmallSort<16>, "insertion: 16");
//! ```
//!
//! ## 2. Annotate families next to their struct definitions
//!
//! ```rust,ignore
//! combo_codegen::family!(
//!     type = QuickSort<{P}, {V}, {SS}>,
//!     uses = [
//!         "super::partitions::{Lomuto, Hoare}",
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
//!     let config = combo_codegen::CodegenConfig::for_sort_families();
//!     let result = combo_codegen::scan("src/", &config).unwrap();
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
pub mod scanner;

pub use family::{
    cross_axis, inline, Axis, AxisSpec, CodegenConfig, Combination, ComponentDef,
    ComponentRegistry, Family, FamilyDef, FieldValue,
};
pub use scanner::{scan, ScanResult};

// ── component! macro ─────────────────────────────────────────────────────────

/// Annotate a concrete type as implementing a named role.
///
/// This macro **expands to nothing**. It exists solely as a marker that the
/// build-script scanner ([`scan`]) recognises and uses to populate a
/// [`ComponentRegistry`].
///
/// # Arguments
///
/// | Position | Meaning | Example |
/// |----------|---------|---------|
/// | 1 | Role identifier | `Partition` |
/// | 2 | Type expression | `InsertionSmallSort<16>` |
/// | 3 | Human-readable label | `"insertion: 16"` |
#[macro_export]
macro_rules! component {
    ($role:ident, $ty:ty, $label:literal) => {};
}

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
