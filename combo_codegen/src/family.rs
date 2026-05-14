use std::collections::HashMap;

// ── ComponentDef ─────────────────────────────────────────────────────────────

/// A single concrete type that fills a role in a generic family.
///
/// `type_expr` is the Rust type as it should appear inside `<…>`, e.g.
/// `"InsertionSmallSort<16>"`. `label` is the human-readable name used in
/// downstream registries, e.g. `"insertion: 16"`.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub type_expr: String,
    pub label: String,
}

impl ComponentDef {
    pub fn new(type_expr: impl Into<String>, label: impl Into<String>) -> Self {
        Self { type_expr: type_expr.into(), label: label.into() }
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

    /// All role names present in the registry, in arbitrary order.
    pub fn roles(&self) -> impl Iterator<Item = &str> {
        self.roles.keys().map(String::as_str)
    }
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
/// → `Bool`, otherwise parsed as `Int`.
#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Bool(bool),
    Int(i64),
    StringArray(Vec<String>),
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
    pub fn render(&self, out: &mut String, registry: &ComponentRegistry, config: &CodegenConfig) {
        let initial_path = self.get_path(config);
        self.render_recursive(out, registry, initial_path, config);
    }

    fn get_path(&self, config: &CodegenConfig) -> Vec<String> {
        let Some(key) = config.path_field.as_deref() else {
            return Vec::new();
        };
        for (k, v) in &self.fields {
            if k == key {
                if let FieldValue::StringArray(a) = v {
                    return a.clone();
                }
            }
        }
        Vec::new()
    }

    fn render_recursive(
        &self,
        out: &mut String,
        registry: &ComponentRegistry,
        path: Vec<String>,
        config: &CodegenConfig,
    ) {
        // Phase 1: split out any Cross-with-extras into two passes — one for
        // the cross-product, one for the extras under a "specialty <role>"
        // sub-branch in the path field.
        let split = self.axes.iter().enumerate().find_map(|(i, (var, spec))| {
            match spec {
                AxisSpec::Cross { extras, left, right, type_tmpl, label_tmpl }
                    if !extras.is_empty() =>
                {
                    let cross_only = AxisSpec::Cross {
                        left: left.clone(),
                        right: right.clone(),
                        type_tmpl: type_tmpl.clone(),
                        label_tmpl: label_tmpl.clone(),
                        extras: Vec::new(),
                    };
                    let extras_only = AxisSpec::Inline(extras.clone());
                    Some((i, var.clone(), left.clone(), cross_only, extras_only))
                }
                _ => None,
            }
        });

        if let Some((idx, var, left_role, cross, extras)) = split {
            let mut main = self.clone();
            main.axes[idx].1 = cross;
            main.render_recursive(out, registry, path.clone(), config);

            let mut spec = self.clone();
            spec.axes[idx].1 = extras;
            let placeholder = format!("{{{var}}}");
            let marker = format!("specialty {}", humanize_role(&left_role));
            let mut new_path = path;
            if let Some(p) = new_path.iter().position(|s| s == &placeholder) {
                new_path.insert(p, marker);
            }
            spec.render_recursive(out, registry, new_path, config);
            return;
        }

        // Phase 2: any remaining Cross axes (extras already stripped) are
        // unrolled into pairs of independent role axes so each side of the
        // cross becomes its own menu level.
        let (transformed, transformed_path) = self.split_crosses_into_role_pairs(&path);
        transformed.render_single(out, registry, &transformed_path, config);
    }

    fn split_crosses_into_role_pairs(&self, path: &[String]) -> (FamilyDef, Vec<String>) {
        let mut type_template = self.type_template.clone();
        let mut new_path = path.to_vec();
        let mut new_axes: Vec<(String, AxisSpec)> = Vec::new();

        for (var, spec) in self.axes.iter().cloned() {
            match spec {
                AxisSpec::Cross { left, right, type_tmpl, .. } => {
                    let left_var = format!("{var}__0");
                    let right_var = format!("{var}__1");

                    let combined = type_tmpl
                        .replace("{0}", &format!("{{{left_var}}}"))
                        .replace("{1}", &format!("{{{right_var}}}"));
                    let placeholder = format!("{{{var}}}");
                    type_template = type_template.replace(&placeholder, &combined);

                    if let Some(p) = new_path.iter().position(|s| s == &placeholder) {
                        new_path.splice(
                            p..p + 1,
                            [format!("{{{left_var}}}"), format!("{{{right_var}}}")],
                        );
                    }

                    new_axes.push((left_var, AxisSpec::Role(left)));
                    new_axes.push((right_var, AxisSpec::Role(right)));
                }
                other => new_axes.push((var, other)),
            }
        }

        let mut new_family = self.clone();
        new_family.type_template = type_template;
        new_family.axes = new_axes;
        (new_family, new_path)
    }

    fn render_single(
        &self,
        out: &mut String,
        registry: &ComponentRegistry,
        path: &[String],
        config: &CodegenConfig,
    ) {
        use std::fmt::Write as _;

        writeln!(out, "{}! {{", config.output_macro).unwrap();
        writeln!(out, "    {}{};", config.type_prefix, self.type_template).unwrap();
        out.push('\n');

        for (var, spec) in &self.axes {
            let components = resolve_axis_spec(spec, registry);
            writeln!(out, "    {var} {{").unwrap();
            for comp in &components {
                writeln!(out, "        {} => \"{}\"", comp.type_expr, comp.label).unwrap();
            }
            out.push_str("    }\n");
        }

        out.push('\n');

        let key_width = self.fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        let path_field = config.path_field.as_deref();

        for (key, value) in &self.fields {
            let rendered = if Some(key.as_str()) == path_field {
                let reordered = reorder_path_by_axis_size(path, &self.axes, registry);
                let inner = reordered
                    .iter()
                    .map(|p| format!("\"{p}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            } else {
                value.render()
            };
            writeln!(out, "    {key:<key_width$} = {rendered};").unwrap();
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

/// Cardinality of an axis after resolving it against `registry`.
fn axis_count(spec: &AxisSpec, registry: &ComponentRegistry) -> usize {
    match spec {
        AxisSpec::Role(role) => registry.role(role).len(),
        AxisSpec::Cross { left, right, extras, .. } => {
            registry.role(left).len() * registry.role(right).len() + extras.len()
        }
        AxisSpec::Inline(items) => items.len(),
    }
}

/// Reorder pure-placeholder path elements so axes with fewer variants come
/// first. Literal elements keep their declared positions; placeholders that
/// don't match a declared axis are also treated as literals.
fn reorder_path_by_axis_size(
    path: &[String],
    axes: &[(String, AxisSpec)],
    registry: &ComponentRegistry,
) -> Vec<String> {
    let positions: Vec<usize> = path
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let slot = e.strip_prefix('{').and_then(|s| s.strip_suffix('}'))?;
            if axes.iter().any(|(n, _)| n == slot) { Some(i) } else { None }
        })
        .collect();

    if positions.len() < 2 {
        return path.to_vec();
    }

    let mut slots: Vec<(&str, usize)> = positions
        .iter()
        .map(|&i| {
            let slot = path[i].strip_prefix('{').unwrap().strip_suffix('}').unwrap();
            let (_, spec) = axes.iter().find(|(n, _)| n == slot).unwrap();
            (slot, axis_count(spec, registry))
        })
        .collect();
    slots.sort_by_key(|&(_, c)| c);

    let mut out = path.to_vec();
    for (&pos, (slot, _)) in positions.iter().zip(slots.iter()) {
        out[pos] = format!("{{{}}}", slot);
    }
    out
}

/// Resolve an [`AxisSpec`] to a concrete list of [`ComponentDef`]s.
fn resolve_axis_spec(spec: &AxisSpec, registry: &ComponentRegistry) -> Vec<ComponentDef> {
    match spec {
        AxisSpec::Role(role) => registry.role(role).to_vec(),
        AxisSpec::Cross { left, right, type_tmpl, label_tmpl, extras } => {
            let tt = type_tmpl.as_str();
            let lt = label_tmpl.as_str();
            let mut result = cross_axis(
                registry.role(left),
                registry.role(right),
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
            ComponentDef::new("Lomuto", "lomuto"),
            ComponentDef::new("Hoare", "hoare"),
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
        assert_eq!(combos[0].instantiated_type(), "QuickSort<Lomuto, FirstElement>");
        assert_eq!(combos[3].instantiated_type(), "QuickSort<Hoare, LastElement>");
    }

    #[test]
    fn get_binding() {
        let family = Family::new("Sort<{P}>").axis("P", &parts());
        let combo = &family.combinations()[0];
        assert_eq!(combo.get("P").unwrap().label, "lomuto");
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
}
