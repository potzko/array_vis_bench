//! The domain seam for entry emission.
//!
//! Stage 4 ([`crate::emit::emit_entries`]) is domain-agnostic: it resolves each
//! spec to a concrete type, wraps it in an import-scoped module exposing the
//! alias `Ty`, and then asks the **backend** for the registration body that goes
//! inside. Everything the compiler emits directly is structural (a module, its
//! `use`s, the type alias); everything domain-specific — the runtime registry
//! record, its inherited complexity, the per-category run/correctness drivers —
//! lives behind this trait, implemented by the consumer's domain (in this
//! prototype, `avb_emit::ArrayBackend`). The compiler never names the ABI, the
//! `AlgorithmEntry` shape, or any `Category`; those are the backend's to know.

/// Per-entry facts the backend needs to render a registration. The resolved
/// algorithm type is in scope at the emission site as the alias `Ty`; the
/// backend references it as `Ty` rather than receiving the type expression, so
/// import scoping stays the compiler's job.
pub struct EntryCtx<'a> {
    /// The resolved label — the entry's stable, human-readable name.
    pub label: &'a str,
    /// The catalog `category` of the root component, verbatim and **opaque** to
    /// the compiler. `None` when the component declared none; the backend
    /// decides what (if anything) that means.
    pub category: Option<&'a str>,
    /// The catalog `adaptive` flag (a per-family literal, not type-inherited).
    pub adaptive: bool,
    /// The catalog `max_input` bound, if any.
    pub max_input: Option<usize>,
}

/// How a domain turns a resolved type into a registry entry. Given the
/// per-entry context (with `Ty` in scope at the emission site), return the Rust
/// source spliced inside the compiler's generated
/// `mod __entry_N { … pub type Ty = …; <RETURNED HERE> }`.
///
/// The returned source must be the module's interior items, indented one level
/// (4 spaces) and newline-terminated, so the compiler can append the closing
/// `}` of the module verbatim.
pub trait EmitBackend {
    fn entry_body(&self, ctx: &EntryCtx) -> Result<String, String>;
}
