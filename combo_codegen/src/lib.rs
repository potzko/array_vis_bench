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
//! ## 2. Annotate sort families next to their struct definitions
//!
//! ```rust,ignore
//! combo_codegen::sort_family!(
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
//! Both macros **expand to nothing** — they are purely markers for the build scanner.
//!
//! ## 3. Scan and generate in `build.rs`
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
//!     let result = combo_codegen::scan("src/").unwrap();
//!     result.emit_rerun();
//!     println!("cargo:rerun-if-changed=build.rs");
//!     result.emit_sort_families(&out_dir).unwrap();
//! }
//! ```
//!
//! ## 4. Consume the generated file
//!
//! ```rust,ignore
//! // quick_sorts/mod.rs
//! pub mod combinations {
//!     include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
//! }
//! ```

pub mod family;
pub mod scanner;

pub use family::{
    Axis, AxisSpec, Combination, ComponentDef, ComponentRegistry, Family, SortFamilyDef,
    cross_axis, inline,
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
///
/// # Example
///
/// ```rust,ignore
/// combo_codegen::component!(Partition, Lomuto, "lomuto");
/// combo_codegen::component!(SmallSort, InsertionSmallSort<16>, "insertion: 16");
/// ```
#[macro_export]
macro_rules! component {
    ($role:ident, $ty:ty, $label:literal) => {};
}

// ── sort_family! macro ────────────────────────────────────────────────────────

/// Declare a sort family inline, next to the sort's struct definition.
///
/// This macro **expands to nothing**. It exists solely as a marker that the
/// build-script scanner ([`scan`]) recognises and uses to generate
/// `*_combinations.rs` files via [`ScanResult::emit_sort_families`].
///
/// # Syntax
///
/// ```rust,ignore
/// combo_codegen::sort_family!(
///     type = SortType<{A}, {B}>,
///     uses = ["path::to::SortType", "path::to::ComponentA"],
///     A: RoleA,                             // simple role axis
///     B: inline [("TypeX", "x"), ("TypeY", "y")],  // inline axis
///     // For cross-product axes:
///     // DPS: cross(Role1, Role2, "Wrapper<{0},{1}>", "{0}/{1}") + [("Extra","extra")],
///     name = "my sort",
///     big_o = "O(N log N)",
///     stable = true,
///     direct_sort = true,
///     path = ["category", "{A}", "{B}"],
/// );
/// ```
#[macro_export]
macro_rules! sort_family {
    ($($tt:tt)*) => {};
}
