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
    /// The catalog `nondeterministic` flag — the backend registers the entry as
    /// determinism-exempt when set.
    pub nondeterministic: bool,
    /// The catalog `max_input` bound, if any.
    pub max_input: Option<usize>,
    /// STRUCTURAL faceted axes (pre-order) derived from the resolved tree: each
    /// type slot contributes a node, immediately followed by a composite filler's
    /// own sub-slots with their `path` prefixed (`pivot`, `pivot/a`, `pivot/b`).
    /// The backend surfaces these for nested picker navigation. Empty for a
    /// slot-less component.
    pub axes: &'a [crate::resolve::AxisNode],
    /// The catalog `menu` sub-path: the structural picker segments *beneath* the
    /// category root (e.g. `["spec", "quick sorts"]`). Empty = register flat at
    /// the root. Purely for navigation placement; never affects dispatch.
    pub menu: &'a [String],
    /// The `label` with each axis rewritten to a `{Role}` hole (see
    /// [`crate::resolve::Resolved::label_template`]). A navigation-aware backend
    /// can register this so the picker renders the partial type in the catalog's
    /// label syntax, filling each hole per axis. Equals `label` (no holes) for a
    /// slot-less component.
    pub label_template: &'a str,
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
