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
//! The macro expands to nothing — it is purely a marker for the build scanner.
//!
//! ## 2. Scan and generate in `build.rs`
//!
//! ```rust,ignore
//! use combo_codegen::{scan, Family, inline};
//!
//! fn main() {
//!     let result = scan("src/").unwrap();
//!     result.emit_rerun();          // cargo:rerun-if-changed for every .rs file
//!
//!     let family = Family::new("QuickSort<{P}, {V}, {SS}>")
//!         .axis("P",  result.registry.role("Partition"))
//!         .axis("V",  result.registry.role("PivotSelector"))
//!         .axis("SS", result.registry.role("SmallSort"));
//!
//!     // Iterate axes to produce whatever output format you need:
//!     let mut code = String::new();
//!     for axis in family.axes() {
//!         for comp in &axis.components {
//!             code += &format!("{} => \"{}\"\n", comp.type_expr, comp.label);
//!         }
//!     }
//!
//!     // Or iterate every combination:
//!     for combo in family.combinations() {
//!         println!("{}", combo.instantiated_type());
//!         // e.g. "QuickSort<Lomuto, FirstElement, NoSmallSort>"
//!     }
//! }
//! ```
//!
//! ## 3. Consume the generated file
//!
//! ```rust,ignore
//! // quick_sorts/mod.rs
//! pub mod combinations {
//!     include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
//! }
//! ```

pub mod family;
pub mod scanner;

pub use family::{Axis, Combination, ComponentDef, ComponentRegistry, Family, cross_axis, inline};
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
