use std::collections::HashMap;

// ── ComponentDef ─────────────────────────────────────────────────────────────

/// A single concrete type that fills a role in a generic sort family.
///
/// `type_expr` is the Rust type as it should appear inside `<…>`, e.g.
/// `"InsertionSmallSort<16>"`. `label` is the human-readable name used in
/// the sort registry path, e.g. `"insertion: 16"`.
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
/// Built by [`crate::scan`]; consumed by [`Family`] to resolve axis definitions.
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
///
/// Useful for axes whose variants are not annotated in source files — for
/// example, boolean const-generic flags:
///
/// ```rust,ignore
/// use combo_codegen::inline;
///
/// Family::new("MySort<{SS}, {PP}>")
///     .axis("SS", registry.role("SmallSort"))
///     .axis("PP", &inline(&[("false", ""), ("true", "ping-pong")]));
/// ```
pub fn inline(items: &[(&str, &str)]) -> Vec<ComponentDef> {
    items
        .iter()
        .map(|(ty, lbl)| ComponentDef::new(*ty, *lbl))
        .collect()
}

/// Compute the cross-product of two component lists, merging each pair into a
/// single [`ComponentDef`] using caller-supplied combiners.
///
/// Useful when a single type-parameter axis represents a *combination* of two
/// independent roles — for example, a `DualPivotSelector` that wraps two
/// independent `PivotSelector` strategies:
///
/// ```rust,ignore
/// let dual = cross_axis(
///     registry.role("PivotSelector"),
///     registry.role("PivotSelector"),
///     |a, b| format!("CombinedSelector<{}, {}>", a.type_expr, b.type_expr),
///     |a, b| format!("{} / {}", a.label, b.label),
/// );
/// ```
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

// ── Axis ─────────────────────────────────────────────────────────────────────

/// One parameter slot in a [`Family`], identified by its template variable
/// name (e.g. `"P"`) and the list of concrete types it can take.
#[derive(Debug, Clone)]
pub struct Axis {
    pub var: String,
    pub components: Vec<ComponentDef>,
}

// ── Family ───────────────────────────────────────────────────────────────────

/// A generic sort type together with its named axes.
///
/// Build with [`Family::new`] and chain [`Family::axis`] calls, then either:
/// - call [`Family::axes`] to iterate axes and format your own output, or
/// - call [`Family::combinations`] to iterate every concrete instantiation.
///
/// # Example
///
/// ```rust,ignore
/// use combo_codegen::{scan, Family, inline};
///
/// let reg = scan("src/").unwrap();
///
/// let family = Family::new("QuickSort<{P}, {V}, {SS}>")
///     .axis("P",  reg.role("Partition"))
///     .axis("V",  reg.role("PivotSelector"))
///     .axis("SS", reg.role("SmallSort"));
///
/// for combo in family.combinations() {
///     println!("{}", combo.instantiated_type());
/// }
/// ```
#[derive(Debug)]
pub struct Family {
    /// Template string with `{VAR}` placeholders, e.g. `"QuickSort<{P}, {V}, {SS}>"`.
    pub type_template: String,
    axes: Vec<Axis>,
}

impl Family {
    /// Create a new family from a type template string.
    ///
    /// Placeholders are written as `{VAR}` where `VAR` matches the first
    /// argument of subsequent `.axis("VAR", …)` calls.
    pub fn new(type_template: impl Into<String>) -> Self {
        Self { type_template: type_template.into(), axes: Vec::new() }
    }

    /// Add an axis bound to the given variable name.
    ///
    /// `components` is typically `registry.role("RoleName")` for discovered
    /// types, or `&inline(&[…])` for hand-written inline variants.
    pub fn axis(mut self, var: impl Into<String>, components: &[ComponentDef]) -> Self {
        self.axes.push(Axis { var: var.into(), components: components.to_vec() });
        self
    }

    /// Iterate the axes in declaration order.
    pub fn axes(&self) -> &[Axis] {
        &self.axes
    }

    /// Instantiate the type template for a given set of axis bindings.
    ///
    /// `bindings` maps variable names to type expressions. Variables not
    /// present in the template are ignored; template variables without a
    /// binding are left as-is.
    pub fn instantiate(&self, bindings: &[(&str, &str)]) -> String {
        let mut result = self.type_template.clone();
        for (var, ty) in bindings {
            result = result.replace(&format!("{{{var}}}"), ty);
        }
        result
    }

    /// Return every combination as a [`Combination`], i.e. the full
    /// cross-product of all axes.
    pub fn combinations(&self) -> Vec<Combination<'_>> {
        // Start with one empty combination and extend it axis by axis.
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

// ── Combination ──────────────────────────────────────────────────────────────

/// One fully-resolved instantiation of a [`Family`].
pub struct Combination<'a> {
    family: &'a Family,
    /// Ordered list of `(var_name, component)` pairs, one per axis.
    pub bindings: Vec<(&'a str, &'a ComponentDef)>,
}

impl<'a> Combination<'a> {
    /// The concrete type string, e.g. `"QuickSort<Lomuto, FirstElement, NoSmallSort>"`.
    pub fn instantiated_type(&self) -> String {
        let pairs: Vec<(&str, &str)> = self
            .bindings
            .iter()
            .map(|(var, comp)| (*var, comp.type_expr.as_str()))
            .collect();
        self.family.instantiate(&pairs)
    }

    /// The value for a specific axis variable, or `None` if not bound.
    pub fn get(&self, var: &str) -> Option<&ComponentDef> {
        self.bindings
            .iter()
            .find(|(v, _)| *v == var)
            .map(|(_, c)| *c)
    }
}

// ── AxisSpec ─────────────────────────────────────────────────────────────────

/// Describes how to populate one axis of a [`SortFamilyDef`].
#[derive(Debug, Clone)]
pub enum AxisSpec {
    /// Populate the axis from a named role in the [`ComponentRegistry`].
    Role(String),
    /// Cross-product of two roles, with optional extra entries appended.
    Cross {
        /// Left role name.
        left: String,
        /// Right role name.
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

// ── SortFamilyDef ─────────────────────────────────────────────────────────────

/// A fully-parsed `sort_family!(…)` annotation found in a source file.
///
/// The annotation declares everything the code generator needs to emit one
/// `sort_registry_macro::sort_family! { … }` invocation.
#[derive(Debug, Clone)]
pub struct SortFamilyDef {
    /// Generic type template with `{VAR}` placeholders, e.g.
    /// `"QuickSort<{P}, {V}, {SS}>"`.
    pub type_template: String,
    /// Axis definitions in declaration order: `(variable_name, spec)`.
    pub axes: Vec<(String, AxisSpec)>,
    /// `use` paths needed in the generated file (without the `use` keyword).
    pub uses: Vec<String>,
    pub name: String,
    pub big_o: String,
    pub stable: bool,
    pub direct_sort: bool,
    /// Path segments; each will be wrapped in `"…"` in the generated code.
    /// Segments may contain `{VAR}` placeholders (e.g. `"{P}"`).
    pub path: Vec<String>,
    /// Parent-directory name of the annotated source file; used to determine
    /// which `*_combinations.rs` file to write (e.g. `"quick_sorts"`).
    pub source_module: String,
}

impl SortFamilyDef {
    /// Resolve axes against `registry` and append one
    /// `sort_registry_macro::sort_family! { … }` block to `out`.
    pub fn render(&self, out: &mut String, registry: &ComponentRegistry) {
        use std::fmt::Write as _;

        out.push_str("sort_registry_macro::sort_family! {\n");
        writeln!(out, "    type Sort = {};", self.type_template).unwrap();
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
        writeln!(out, "    name        = \"{}\";", self.name).unwrap();
        writeln!(out, "    big_o       = \"{}\";", self.big_o).unwrap();
        writeln!(out, "    stable      = {};", self.stable).unwrap();
        writeln!(out, "    direct_sort = {};", self.direct_sort).unwrap();

        let path_str = self
            .path
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "    path        = [{path_str}];").unwrap();

        out.push_str("}\n\n");
    }
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
}
