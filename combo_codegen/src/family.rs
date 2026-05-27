use std::collections::HashMap;

// ── Slot ─────────────────────────────────────────────────────────────────────

/// A recursive parameter slot on a composite [`ComponentDef`]. `param` is the
/// `{param}` placeholder inside the component's `type_expr` / `label`; `role`
/// is the registry role whose components may fill it. Expansion (see
/// [`expand_role`]) recursively fills slots, bounded by the trail rule.
#[derive(Debug, Clone)]
pub struct Slot {
    pub param: String,
    pub role: String,
}

impl Slot {
    pub fn new(param: impl Into<String>, role: impl Into<String>) -> Self {
        Self { param: param.into(), role: role.into() }
    }
}

// ── ComponentDef ─────────────────────────────────────────────────────────────

/// A single concrete type that fills a role in a generic family.
///
/// `type_expr` is the Rust type as it should appear inside `<…>`, e.g.
/// `"InsertionSmallSort<16>"`. `label` is the human-readable name used in
/// downstream registries, e.g. `"insertion: 16"`.
///
/// When `slots` is non-empty the component is *composite*: `type_expr` and
/// `label` carry `{param}` placeholders that [`expand_role`] fills from other
/// roles, recursively. A leaf component has an empty `slots`.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub type_expr: String,
    pub label: String,
    /// `use` paths this component needs in the generated file (its own type
    /// plus any generic-argument types). Unioned per family at emit time so
    /// families no longer have to list component imports themselves.
    pub uses: Vec<String>,
    /// Recursive parameter slots. Empty for leaf components (the common case).
    pub slots: Vec<Slot>,
}

impl ComponentDef {
    pub fn new(type_expr: impl Into<String>, label: impl Into<String>) -> Self {
        Self { type_expr: type_expr.into(), label: label.into(), uses: Vec::new(), slots: Vec::new() }
    }

    pub fn with_uses(
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
    ) -> Self {
        Self { type_expr: type_expr.into(), label: label.into(), uses, slots: Vec::new() }
    }

    pub fn with_uses_and_slots(
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
        slots: Vec<Slot>,
    ) -> Self {
        Self { type_expr: type_expr.into(), label: label.into(), uses, slots }
    }
}

// ── ComponentRegistry ────────────────────────────────────────────────────────

/// Maps role names (e.g. `"Partition"`) to their discovered [`ComponentDef`]s.
///
/// Built by [`crate::scan`]; consumed by [`Family`] / [`FamilyDef`] to resolve
/// axis definitions.
#[derive(Debug, Default)]
pub struct ComponentRegistry {
    roles: HashMap<String, Vec<ComponentDef>>,
}

impl ComponentRegistry {
    /// Return every component registered under `role`, in discovery order.
    /// Returns an empty slice if the role is unknown.
    pub fn role(&self, role: &str) -> &[ComponentDef] {
        self.roles.get(role).map_or(&[], Vec::as_slice)
    }

    /// Register a component. Called by the scanner; also available for manual
    /// use in `build.rs` if you want to mix scanned and hand-written entries.
    pub fn add(
        &mut self,
        role: impl Into<String>,
        type_expr: impl Into<String>,
        label: impl Into<String>,
    ) {
        self.roles
            .entry(role.into())
            .or_default()
            .push(ComponentDef::new(type_expr, label));
    }

    /// Insert a component at the front of its role list. The build script
    /// uses this when merging components from a higher-priority source
    /// (Cargo.toml metadata) into a registry already populated by the text
    /// scanner. Iterating metadata in *reverse* declaration order and
    /// calling `add_front` for each leaves them at the front in original
    /// declaration order.
    pub fn add_front(
        &mut self,
        role: impl Into<String>,
        type_expr: impl Into<String>,
        label: impl Into<String>,
    ) {
        self.roles
            .entry(role.into())
            .or_default()
            .insert(0, ComponentDef::new(type_expr, label));
    }

    /// Like [`add_front`](Self::add_front) but also records the component's
    /// own `use` paths, which the emitter unions into each consuming family.
    pub fn add_front_with_uses(
        &mut self,
        role: impl Into<String>,
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
    ) {
        self.roles
            .entry(role.into())
            .or_default()
            .insert(0, ComponentDef::with_uses(type_expr, label, uses));
    }

    /// Like [`add_front_with_uses`](Self::add_front_with_uses) but also records
    /// recursive [`Slot`]s, making the component composite. Used by the
    /// metadata scanner to register components generic over a role.
    pub fn add_front_full(
        &mut self,
        role: impl Into<String>,
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
        slots: Vec<Slot>,
    ) {
        self.roles
            .entry(role.into())
            .or_default()
            .insert(0, ComponentDef::with_uses_and_slots(type_expr, label, uses, slots));
    }


    /// All role names present in the registry, in arbitrary order.
    pub fn roles(&self) -> impl Iterator<Item = &str> {
        self.roles.keys().map(String::as_str)
    }
}

// ── Recursive expansion (trail rule) ──────────────────────────────────────────

/// Head identifier of a type expression — everything before the first `<`,
/// trimmed. `"QuickBuild<{P}>"` → `"QuickBuild"`, `"Lomuto"` → `"Lomuto"`.
/// Used as the type identity in trail edges.
fn type_head(type_expr: &str) -> &str {
    type_expr.split('<').next().unwrap_or(type_expr).trim()
}

/// One step on a composition path: `(parent head, slot param, child head)`.
/// The trail rule forbids the same edge appearing twice on a single
/// root→leaf path — this is what makes a recursive role graph enumerable.
type Edge = (String, String, String);

/// Expand every component registered under `role` into concrete, slot-free
/// [`ComponentDef`]s, recursively filling composite slots from the registry.
///
/// Recursion is bounded by the **trail rule**: along any single root→leaf
/// path, the same `(parent, slot, child)` edge may not be reused. A type may
/// recur in a slot if it arrives via a different parent; only the exact edge
/// is pruned. Termination follows from edge-exhaustion — a path is a trail, so
/// its length is bounded by the (finite) number of distinct edges.
///
/// Leaf components (empty `slots`) expand to themselves, so a registry with no
/// composite components returns each role's list unchanged.
pub fn expand_role(registry: &ComponentRegistry, role: &str) -> Vec<ComponentDef> {
    let mut out = Vec::new();
    for comp in registry.role(role) {
        expand_component(registry, comp, &mut Vec::new(), &mut out);
    }
    out
}

/// Expand one component, appending its concrete instantiations to `out`.
/// `used_edges` is the set of trail edges already taken on the path to here
/// (push-on-enter / pop-on-exit, so it stays per-path).
fn expand_component(
    registry: &ComponentRegistry,
    comp: &ComponentDef,
    used_edges: &mut Vec<Edge>,
    out: &mut Vec<ComponentDef>,
) {
    if comp.slots.is_empty() {
        out.push(comp.clone());
        return;
    }

    let parent_head = type_head(&comp.type_expr).to_string();

    // For each slot, the concrete child options legal at this point on the
    // path (trail-pruned, then recursively expanded to leaves).
    let mut slot_options: Vec<Vec<ComponentDef>> = Vec::with_capacity(comp.slots.len());
    for slot in &comp.slots {
        let mut opts = Vec::new();
        for child in registry.role(&slot.role) {
            let edge = (
                parent_head.clone(),
                slot.param.clone(),
                type_head(&child.type_expr).to_string(),
            );
            if used_edges.contains(&edge) {
                continue; // trail rule: this exact edge is already on the path
            }
            used_edges.push(edge);
            expand_component(registry, child, used_edges, &mut opts);
            used_edges.pop();
        }
        slot_options.push(opts);
    }

    // Cartesian product across slots → one concrete ComponentDef per combo,
    // substituting `{param}` in both type_expr and label and unioning uses.
    for combo in cartesian(&slot_options) {
        let mut type_expr = comp.type_expr.clone();
        let mut label = comp.label.clone();
        let mut uses = comp.uses.clone();
        for (slot, chosen) in comp.slots.iter().zip(&combo) {
            let ph = format!("{{{}}}", slot.param);
            type_expr = type_expr.replace(&ph, &chosen.type_expr);
            label = label.replace(&ph, &chosen.label);
            for u in &chosen.uses {
                if !uses.contains(u) {
                    uses.push(u.clone());
                }
            }
        }
        out.push(ComponentDef { type_expr, label, uses, slots: Vec::new() });
    }
}

/// Cartesian product of per-slot option lists. Returns one `Vec` of chosen
/// components per combination. An empty option list (every child pruned)
/// collapses the product to nothing — that path simply yields no variants.
fn cartesian<'a>(slot_options: &'a [Vec<ComponentDef>]) -> Vec<Vec<&'a ComponentDef>> {
    let mut combos: Vec<Vec<&ComponentDef>> = vec![Vec::new()];
    for opts in slot_options {
        let mut next = Vec::with_capacity(combos.len() * opts.len());
        for combo in &combos {
            for opt in opts {
                let mut extended = combo.clone();
                extended.push(opt);
                next.push(extended);
            }
        }
        combos = next;
    }
    combos
}

// ── Axis helpers ─────────────────────────────────────────────────────────────

/// Build a `Vec<ComponentDef>` from a slice of `(type_expr, label)` pairs.
pub fn inline(items: &[(&str, &str)]) -> Vec<ComponentDef> {
    items
        .iter()
        .map(|(ty, lbl)| ComponentDef::new(*ty, *lbl))
        .collect()
}

/// Cross-product of two component lists, merging each pair into a single
/// [`ComponentDef`] using caller-supplied combiners.
pub fn cross_axis(
    left: &[ComponentDef],
    right: &[ComponentDef],
    type_fn: impl Fn(&ComponentDef, &ComponentDef) -> String,
    label_fn: impl Fn(&ComponentDef, &ComponentDef) -> String,
) -> Vec<ComponentDef> {
    let mut out = Vec::with_capacity(left.len() * right.len());
    for l in left {
        for r in right {
            out.push(ComponentDef::new(type_fn(l, r), label_fn(l, r)));
        }
    }
    out
}

// ── Axis (manual API) ────────────────────────────────────────────────────────

/// One parameter slot in a [`Family`].
#[derive(Debug, Clone)]
pub struct Axis {
    pub var: String,
    pub components: Vec<ComponentDef>,
}

// ── Family / Combination (manual API) ────────────────────────────────────────

/// A generic type together with its named axes, built imperatively.
#[derive(Debug)]
pub struct Family {
    pub type_template: String,
    axes: Vec<Axis>,
}

impl Family {
    pub fn new(type_template: impl Into<String>) -> Self {
        Self { type_template: type_template.into(), axes: Vec::new() }
    }

    pub fn axis(mut self, var: impl Into<String>, components: &[ComponentDef]) -> Self {
        self.axes.push(Axis { var: var.into(), components: components.to_vec() });
        self
    }

    pub fn axes(&self) -> &[Axis] {
        &self.axes
    }

    pub fn instantiate(&self, bindings: &[(&str, &str)]) -> String {
        let mut result = self.type_template.clone();
        for (var, ty) in bindings {
            result = result.replace(&format!("{{{var}}}"), ty);
        }
        result
    }

    pub fn combinations(&self) -> Vec<Combination<'_>> {
        let mut combos: Vec<Vec<(&str, &ComponentDef)>> = vec![vec![]];

        for axis in &self.axes {
            let mut next = Vec::with_capacity(combos.len() * axis.components.len());
            for combo in &combos {
                for comp in &axis.components {
                    let mut extended = combo.clone();
                    extended.push((axis.var.as_str(), comp));
                    next.push(extended);
                }
            }
            combos = next;
        }

        combos
            .into_iter()
            .map(|bindings| Combination { family: self, bindings })
            .collect()
    }
}

/// One fully-resolved instantiation of a [`Family`].
pub struct Combination<'a> {
    family: &'a Family,
    pub bindings: Vec<(&'a str, &'a ComponentDef)>,
}

impl<'a> Combination<'a> {
    pub fn instantiated_type(&self) -> String {
        let pairs: Vec<(&str, &str)> = self
            .bindings
            .iter()
            .map(|(var, comp)| (*var, comp.type_expr.as_str()))
            .collect();
        self.family.instantiate(&pairs)
    }

    pub fn get(&self, var: &str) -> Option<&ComponentDef> {
        self.bindings
            .iter()
            .find(|(v, _)| *v == var)
            .map(|(_, c)| *c)
    }
}

// ── AxisSpec ─────────────────────────────────────────────────────────────────

/// How to populate one axis of a scanned [`FamilyDef`].
#[derive(Debug, Clone)]
pub enum AxisSpec {
    /// Populate the axis from a named role in the [`ComponentRegistry`].
    Role(String),
    /// Cross-product of two roles, with optional extra entries appended.
    Cross {
        left: String,
        right: String,
        /// Type-expression template; `{0}` and `{1}` are the pair's type exprs.
        type_tmpl: String,
        /// Label template; `{0}` and `{1}` are the pair's labels.
        label_tmpl: String,
        /// Entries appended after the cross-product.
        extras: Vec<ComponentDef>,
    },
    /// A hand-written list of `(type_expr, label)` pairs.
    Inline(Vec<ComponentDef>),
}

// ── FieldValue ───────────────────────────────────────────────────────────────

/// A loosely-typed value emitted as `key = value;` in the generated output.
///
/// The scanner classifies each trailing `key = value` pair from a `family!(…)`
/// (or `sort_family!(…)`) body into one of these variants based on the value's
/// shape: leading `"` → `String`, leading `[` → `StringArray`, `true`/`false`
/// → `Bool`, bare identifier (e.g. `inherited`) → `Ident`, otherwise
/// parsed as `Int`.
#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Bool(bool),
    Int(i64),
    StringArray(Vec<String>),
    /// A bare identifier passed through verbatim. Used for keywords like
    /// `inherited` that the downstream consumer macro interprets — the
    /// scanner doesn't need to know what they mean, just preserve them.
    Ident(String),
}

impl FieldValue {
    /// Render as it should appear on the right-hand side of `key = …;`.
    fn render(&self) -> String {
        match self {
            FieldValue::String(s) => format!("\"{}\"", s),
            FieldValue::Bool(b) => b.to_string(),
            FieldValue::Int(n) => n.to_string(),
            FieldValue::StringArray(a) => {
                let inner = a
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            }
            FieldValue::Ident(s) => s.clone(),
        }
    }
}

// ── CodegenConfig ────────────────────────────────────────────────────────────

/// Knobs that bind the generic family scanner+emitter to a specific consumer
/// macro (e.g. `sort_registry_macro::sort_family!`).
///
/// Construct via [`CodegenConfig::new`] + builder methods, or use the
/// [`CodegenConfig::for_sort_families`] preset for the existing sort use case.
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Source-side marker the scanner looks for, e.g. `"sort_family"`.
    /// The scanner matches the literal `"<marker>!("` anywhere in `.rs` files.
    pub marker: String,
    /// Macro path written into the generated file, e.g.
    /// `"sort_registry_macro::sort_family"` — emitted as
    /// `<output_macro>! { … }`.
    pub output_macro: String,
    /// Literal text placed before the family's type template inside the macro
    /// body, e.g. `"type Sort = "`. Set to `""` if the consumer macro doesn't
    /// expect a leading keyword.
    pub type_prefix: String,
    /// Output filename suffix combined with the source module name, e.g.
    /// `"_combinations.rs"` produces `<module>_combinations.rs`.
    pub filename_suffix: String,
    /// Optional name of a `StringArray` field that should receive
    /// menu-path-style transformations (smallest-axis-first reorder,
    /// cross-with-extras "specialty" sub-branch grouping). If `None`, those
    /// transformations are skipped.
    pub path_field: Option<String>,
}

impl CodegenConfig {
    pub fn new(marker: impl Into<String>, output_macro: impl Into<String>) -> Self {
        Self {
            marker: marker.into(),
            output_macro: output_macro.into(),
            type_prefix: String::new(),
            filename_suffix: "_combinations.rs".into(),
            path_field: None,
        }
    }

    pub fn with_type_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.type_prefix = prefix.into();
        self
    }

    pub fn with_filename_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.filename_suffix = suffix.into();
        self
    }

    pub fn with_path_field(mut self, field: impl Into<String>) -> Self {
        self.path_field = Some(field.into());
        self
    }

    /// Preset matching the existing `sort_registry_macro::sort_family!` consumer.
    pub fn for_sort_families() -> Self {
        Self::new("family", "sort_registry_macro::sort_family")
            .with_type_prefix("type Sort = ")
            .with_path_field("path")
    }
}

// ── FamilyDef ────────────────────────────────────────────────────────────────

/// A fully-parsed `family!(…)` (or `sort_family!(…)`) annotation.
///
/// `fields` preserves declaration order. The optional path-field — named via
/// [`CodegenConfig::path_field`] — receives the menu-path transformations
/// (smallest-axis-first reorder, cross-with-extras "specialty" grouping).
#[derive(Debug, Clone)]
pub struct FamilyDef {
    /// Generic type template with `{VAR}` placeholders, e.g.
    /// `"QuickSort<{P}, {V}, {SS}>"`.
    pub type_template: String,
    /// Axis definitions in declaration order: `(variable_name, spec)`.
    pub axes: Vec<(String, AxisSpec)>,
    /// `use` paths needed in the generated file (without the `use` keyword).
    pub uses: Vec<String>,
    /// Trailing `key = value` fields in declaration order.
    pub fields: Vec<(String, FieldValue)>,
    /// Parent-directory name of the annotated source file; determines which
    /// `<module><filename_suffix>` file to write.
    pub source_module: String,
}

impl FamilyDef {
    /// Resolve axes against `registry` and append `<config.output_macro>!`
    /// blocks to `out`.
    ///
    /// Two structural transforms happen here (both conditional on
    /// [`CodegenConfig::path_field`] for the path manipulation parts):
    ///
    /// 1. **Cross-with-extras splits.** A `cross(...) + extras` axis is
    ///    rendered as two separate blocks: one with just the cross-product,
    ///    one with just the extras. The extras block gets a
    ///    `"specialty <role>"` literal inserted before its axis placeholder in
    ///    the path field so it surfaces as a sibling sub-branch.
    ///
    /// 2. **Smallest-axis-first path reorder.** Pure-placeholder elements of
    ///    the path field are reordered by their axis cardinality so each menu
    ///    step branches on the smallest available axis. Literals stay pinned.
    /// Resolve axes against `registry` and append one `<config.output_macro>!`
    /// block to `out`.
    ///
    /// Each axis is emitted with its humanized **role** (`{var} : "role" {…}`)
    /// so the consumer macro can tag every faceted slot. `Cross` axes are
    /// resolved to a single flat list (cross-product + extras), keeping the
    /// `{var}` placeholder intact in the type template — the consumer fills
    /// it with the whole combined type. The `path` field is emitted verbatim;
    /// the consumer derives the menu structure (category vs faceted axis) from
    /// it plus the per-axis roles.
    pub fn render(&self, out: &mut String, registry: &ComponentRegistry, config: &CodegenConfig) {
        self.render_single(out, registry, config);
    }

    /// Humanized role label for an axis, or `""` for `Inline` axes (which the
    /// consumer treats as structural category segments, not faceted axes).
    fn axis_role(spec: &AxisSpec) -> String {
        match spec {
            AxisSpec::Role(role) => humanize_role(role),
            // A cross axis is a *pair* (e.g. two pivot selectors). Give it a
            // role distinct from the single-value role so the faceted picker
            // doesn't pool single- and dual-pivot selectors into one list.
            AxisSpec::Cross { left, .. } => format!("{} pair", humanize_role(left)),
            AxisSpec::Inline(_) => String::new(),
        }
    }

    fn render_single(
        &self,
        out: &mut String,
        registry: &ComponentRegistry,
        config: &CodegenConfig,
    ) {
        use std::fmt::Write as _;

        writeln!(out, "{}! {{", config.output_macro).unwrap();
        writeln!(out, "    {}{};", config.type_prefix, self.type_template).unwrap();
        out.push('\n');

        for (var, spec) in &self.axes {
            let components = resolve_axis_spec(spec, registry);
            let role = Self::axis_role(spec);
            if role.is_empty() {
                writeln!(out, "    {var} {{").unwrap();
            } else {
                writeln!(out, "    {var} : \"{role}\" {{").unwrap();
            }
            for comp in &components {
                writeln!(out, "        {} => \"{}\"", comp.type_expr, comp.label).unwrap();
            }
            out.push_str("    }\n");
        }

        out.push('\n');

        let key_width = self.fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

        for (key, value) in &self.fields {
            writeln!(out, "    {key:<key_width$} = {};", value.render()).unwrap();
        }

        out.push_str("}\n\n");
    }
}

/// Convert a Rust trait name like `PivotSelector` to `"pivot selector"` —
/// inserts a space before each interior uppercase letter and lowercases.
fn humanize_role(role: &str) -> String {
    let mut out = String::new();
    for (i, c) in role.char_indices() {
        if c.is_uppercase() && i > 0 {
            out.push(' ');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Resolve an [`AxisSpec`] to a concrete list of [`ComponentDef`]s.
fn resolve_axis_spec(spec: &AxisSpec, registry: &ComponentRegistry) -> Vec<ComponentDef> {
    match spec {
        AxisSpec::Role(role) => expand_role(registry, role),
        AxisSpec::Cross { left, right, type_tmpl, label_tmpl, extras } => {
            let tt = type_tmpl.as_str();
            let lt = label_tmpl.as_str();
            let mut result = cross_axis(
                &expand_role(registry, left),
                &expand_role(registry, right),
                |l, r| tt.replace("{0}", &l.type_expr).replace("{1}", &r.type_expr),
                |l, r| lt.replace("{0}", &l.label).replace("{1}", &r.label),
            );
            result.extend_from_slice(extras);
            result
        }
        AxisSpec::Inline(items) => items.clone(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> Vec<ComponentDef> {
        vec![
            ComponentDef::new("LeftLeftPartition", "left-left pointer"),
            ComponentDef::new("LeftRightPartition", "left-right pointer"),
        ]
    }

    fn pivots() -> Vec<ComponentDef> {
        vec![
            ComponentDef::new("FirstElement", "first"),
            ComponentDef::new("LastElement", "last"),
        ]
    }

    #[test]
    fn combination_count() {
        let family = Family::new("Sort<{P}, {V}>")
            .axis("P", &parts())
            .axis("V", &pivots());
        assert_eq!(family.combinations().len(), 4); // 2 × 2
    }

    #[test]
    fn instantiated_type() {
        let family = Family::new("QuickSort<{P}, {V}>")
            .axis("P", &parts())
            .axis("V", &pivots());
        let combos = family.combinations();
        assert_eq!(combos[0].instantiated_type(), "QuickSort<LeftLeftPartition, FirstElement>");
        assert_eq!(combos[3].instantiated_type(), "QuickSort<LeftRightPartition, LastElement>");
    }

    #[test]
    fn get_binding() {
        let family = Family::new("Sort<{P}>").axis("P", &parts());
        let combo = &family.combinations()[0];
        assert_eq!(combo.get("P").unwrap().label, "left-left pointer");
        assert!(combo.get("X").is_none());
    }

    #[test]
    fn inline_helper() {
        let bools = inline(&[("false", ""), ("true", "ping-pong")]);
        assert_eq!(bools.len(), 2);
        assert_eq!(bools[1].type_expr, "true");
        assert_eq!(bools[1].label, "ping-pong");
    }

    #[test]
    fn registry_add_and_role() {
        let mut reg = ComponentRegistry::default();
        reg.add("Foo", "Bar", "bar");
        reg.add("Foo", "Baz", "baz");
        assert_eq!(reg.role("Foo").len(), 2);
        assert_eq!(reg.role("Unknown").len(), 0);
    }

    #[test]
    fn field_value_render() {
        assert_eq!(FieldValue::String("hi".into()).render(), "\"hi\"");
        assert_eq!(FieldValue::Bool(true).render(), "true");
        assert_eq!(FieldValue::Int(42).render(), "42");
        assert_eq!(
            FieldValue::StringArray(vec!["a".into(), "b".into()]).render(),
            "[\"a\", \"b\"]"
        );
    }

    #[test]
    fn config_sort_preset() {
        let c = CodegenConfig::for_sort_families();
        assert_eq!(c.marker, "family");
        assert_eq!(c.output_macro, "sort_registry_macro::sort_family");
        assert_eq!(c.type_prefix, "type Sort = ");
        assert_eq!(c.path_field.as_deref(), Some("path"));
    }

    // ── Recursive expansion / trail rule ───────────────────────────────────────

    #[test]
    fn expand_role_leaves_passthrough() {
        // A registry with no composite components returns each role unchanged.
        let mut reg = ComponentRegistry::default();
        reg.add("Partition", "Lomuto", "lomuto");
        reg.add("Partition", "Hoare", "hoare");
        let out = expand_role(&reg, "Partition");
        let types: Vec<&str> = out.iter().map(|c| c.type_expr.as_str()).collect();
        assert_eq!(types, vec!["Lomuto", "Hoare"]);
    }

    /// The mutually-recursive demo graph:
    ///   Partition  = { Lomuto, Hoare, HeapExtract<{B}: HeapBuild> }
    ///   HeapBuild  = { SimpleBuild, QuickBuild<{P}: Partition> }
    /// Without the trail rule this loops forever
    /// (HeapExtract→QuickBuild→HeapExtract→…).
    fn recursive_registry() -> ComponentRegistry {
        let mut reg = ComponentRegistry::default();
        reg.add("Partition", "Lomuto", "lomuto");
        reg.add("Partition", "Hoare", "hoare");
        reg.roles
            .get_mut("Partition")
            .unwrap()
            .push(ComponentDef::with_uses_and_slots(
                "HeapExtract<{B}>",
                "heap extract<{B}>",
                Vec::new(),
                vec![Slot::new("B", "HeapBuild")],
            ));
        reg.add("HeapBuild", "SimpleBuild", "simple build");
        reg.roles
            .get_mut("HeapBuild")
            .unwrap()
            .push(ComponentDef::with_uses_and_slots(
                "QuickBuild<{P}>",
                "quick build<{P}>",
                Vec::new(),
                vec![Slot::new("P", "Partition")],
            ));
        reg
    }

    #[test]
    fn expand_role_trail_bounded_terminates_with_expected_set() {
        let reg = recursive_registry();
        let out = expand_role(&reg, "Partition");
        let types: Vec<&str> = out.iter().map(|c| c.type_expr.as_str()).collect();
        // Six variants: two leaves, then HeapExtract over each non-looping
        // HeapBuild. The deepest re-nest (variant 6) is forced to bottom out
        // in SimpleBuild because the (HeapExtract, B, QuickBuild) edge is
        // already on the path.
        assert_eq!(
            types,
            vec![
                "Lomuto",
                "Hoare",
                "HeapExtract<SimpleBuild>",
                "HeapExtract<QuickBuild<Lomuto>>",
                "HeapExtract<QuickBuild<Hoare>>",
                "HeapExtract<QuickBuild<HeapExtract<SimpleBuild>>>",
            ],
        );
    }

    #[test]
    fn expand_role_prunes_the_looping_edge() {
        let reg = recursive_registry();
        let types: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.type_expr).collect();
        // The looped shape — QuickBuild nested inside a HeapExtract that is
        // itself inside a QuickBuild — must never appear: that repeats the
        // (HeapExtract, B, QuickBuild) edge.
        assert!(
            !types.iter().any(|t| t.contains("QuickBuild<HeapExtract<QuickBuild")),
            "looped edge leaked: {types:?}",
        );
        // Labels are templated too.
        let labels: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.label).collect();
        assert!(labels.contains(&"heap extract<quick build<lomuto>>".to_string()));
    }

    #[test]
    fn expand_role_unions_child_uses() {
        let mut reg = ComponentRegistry::default();
        reg.add_front_full(
            "Partition",
            "HeapExtract<{B}>",
            "heap extract<{B}>",
            vec!["demo::HeapExtract".to_string()],
            vec![Slot::new("B", "HeapBuild")],
        );
        reg.add_front_with_uses(
            "HeapBuild",
            "SimpleBuild",
            "simple build",
            vec!["demo::SimpleBuild".to_string()],
        );
        let out = expand_role(&reg, "Partition");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].type_expr, "HeapExtract<SimpleBuild>");
        assert!(out[0].uses.contains(&"demo::HeapExtract".to_string()));
        assert!(out[0].uses.contains(&"demo::SimpleBuild".to_string()));
    }
}
