use std::collections::HashMap;

// ── Slot ─────────────────────────────────────────────────────────────────────

/// A recursive parameter slot on a composite [`ComponentDef`]. `param` is the
/// `{param}` placeholder inside the component's `type_expr` / `label`; `role`
/// is the registry role whose components may fill it. Expansion (see
/// [`expand_role`]) recursively fills slots, bounded by the head-count rule.
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
    /// Menu navigation segments auto-derived from `label`. A leaf label like
    /// `"binary"` produces `["binary"]`; a composite label like
    /// `"heap extract<{A}, {DH}>"` produces `["heap extract", "{A}", "{DH}"]`,
    /// whose `{param}` segments are spliced with the chosen child's
    /// `path_segments` during recursive expansion. Used by the family renderer
    /// to tier each composite slot into its own menu step.
    pub path_segments: Vec<String>,
}

impl ComponentDef {
    pub fn new(type_expr: impl Into<String>, label: impl Into<String>) -> Self {
        let label = label.into();
        let path_segments = auto_path_segments(&label);
        Self { type_expr: type_expr.into(), label, uses: Vec::new(), slots: Vec::new(), path_segments }
    }

    pub fn with_uses(
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
    ) -> Self {
        let label = label.into();
        let path_segments = auto_path_segments(&label);
        Self { type_expr: type_expr.into(), label, uses, slots: Vec::new(), path_segments }
    }

    pub fn with_uses_and_slots(
        type_expr: impl Into<String>,
        label: impl Into<String>,
        uses: Vec<String>,
        slots: Vec<Slot>,
    ) -> Self {
        let label = label.into();
        let path_segments = auto_path_segments(&label);
        Self { type_expr: type_expr.into(), label, uses, slots, path_segments }
    }
}

/// Auto-derive [`ComponentDef::path_segments`] from a label.
///
/// Convention: split on the first top-level `<` / matching `>` pair, with the
/// inner part split by top-level commas. So `"heap extract<{A}, {DH}>"` →
/// `["heap extract", "{A}", "{DH}"]`. Labels without `<` map to a single
/// segment (`["binary"]` for `"binary"`). Malformed labels (mismatched
/// brackets) fall back to a single segment.
pub fn auto_path_segments(label: &str) -> Vec<String> {
    let label = label.trim();
    let Some(open) = label.find('<') else {
        return vec![label.to_string()];
    };
    let head = label[..open].trim().to_string();
    let bytes = label.as_bytes();
    let mut depth = 1;
    let mut close = label.len();
    let mut i = open + 1;
    while i < label.len() {
        match bytes[i] as char {
            '<' | '{' | '[' | '(' => depth += 1,
            '>' | '}' | ']' | ')' => {
                depth -= 1;
                if depth == 0 {
                    close = i;
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        return vec![label.to_string()];
    }
    let inner = &label[open + 1..close];
    let mut segments = vec![head];
    segments.extend(split_top_level_args(inner));
    segments
}

/// Split a comma-separated argument list at top level only, respecting
/// nested `<…>` / `(…)` / `[…]` / `{…}` pairs. Empty parts are dropped.
fn split_top_level_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '{' | '[' | '(' => depth += 1,
            '>' | '}' | ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                let seg = s[start..i].trim();
                if !seg.is_empty() {
                    out.push(seg.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = s[start..].trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

// ── Alias-substitution helpers ───────────────────────────────────────────────

/// Expand grouped `use` paths into one entry per imported name.
///
/// Metadata entries like `"heap_sort_lib::heap_sort::{HeapSort, NaryHeapSort}"`
/// are split into `"heap_sort_lib::heap_sort::HeapSort"` and
/// `"heap_sort_lib::heap_sort::NaryHeapSort"`. Plain (non-grouped) paths pass
/// through unchanged. The aliaser needs one path per *target name* so it can
/// assign each its own `use … as …;` line.
pub fn expand_grouped_uses(uses: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for raw in uses {
        let trimmed = raw.trim();
        if let Some(open) = trimmed.find('{') {
            let Some(close) = trimmed.rfind('}') else {
                out.push(trimmed.to_string());
                continue;
            };
            let prefix = trimmed[..open].trim_end();
            let inner = &trimmed[open + 1..close];
            for name in inner.split(',') {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                out.push(format!("{prefix}{name}"));
            }
        } else {
            out.push(trimmed.to_string());
        }
    }
    out
}

/// Last `::`-separated segment of a path — the imported short name. Used as
/// the basename for alias generation and as the identifier to substitute in
/// `type_expr` strings.
pub fn use_short_name(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path).trim()
}

/// Assign a deterministic, unique alias to every input path. Aliases are
/// `<short_name>_<N>` where `N` increments per short name in path-sorted
/// order, so two different paths sharing a short name (the collision case)
/// land at `LeftLeftPartition_0` and `LeftLeftPartition_1`, and pure
/// singletons still get `_0` for a uniform shape across the generated file.
pub fn build_alias_map(paths: &[String]) -> HashMap<String, String> {
    let mut unique: Vec<String> = paths.iter().cloned().collect();
    unique.sort();
    unique.dedup();

    let mut counters: HashMap<String, usize> = HashMap::new();
    let mut out = HashMap::new();
    for path in &unique {
        let short = use_short_name(path);
        let idx = counters.entry(short.to_string()).or_insert(0);
        let alias = format!("{short}_{}", *idx);
        *idx += 1;
        out.insert(path.clone(), alias);
    }
    out
}

/// For a single component's `uses`, build a `short_name → alias` map by
/// looking up each path in the module-level [`build_alias_map`]. The
/// substitution helper consumes this to rewrite a component's `type_expr` so
/// each identifier becomes the alias of the path the component's `uses`
/// declared.
pub fn local_alias_map(
    uses: &[String],
    global: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for path in expand_grouped_uses(uses) {
        if let Some(alias) = global.get(&path) {
            out.insert(use_short_name(&path).to_string(), alias.clone());
        }
    }
    out
}

/// Replace every identifier in `input` that matches a key in `map` with the
/// corresponding alias. Identifiers are runs of `[A-Za-z0-9_]`; anything
/// else (`<`, `>`, `,`, whitespace, `{`, `}`, etc.) is a boundary.
///
/// Numeric literals like `16` in `InsertionSmallSort<LinearInsertion, 16>`
/// are technically identifier-shaped runs but won't appear as map keys
/// (you can't `use foo::16`), so they pass through unchanged.
pub fn substitute_aliases(input: &str, map: &HashMap<String, String>) -> String {
    if map.is_empty() {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut ident = String::new();
    let flush = |ident: &mut String, out: &mut String, map: &HashMap<String, String>| {
        if !ident.is_empty() {
            if let Some(alias) = map.get(ident.as_str()) {
                out.push_str(alias);
            } else {
                out.push_str(ident);
            }
            ident.clear();
        }
    };
    for c in input.chars() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
        } else {
            flush(&mut ident, &mut out, map);
            out.push(c);
        }
    }
    flush(&mut ident, &mut out, map);
    out
}

// ── ComponentRegistry ────────────────────────────────────────────────────────

/// Maps role names (e.g. `"Partition"`) to their discovered [`ComponentDef`]s.
///
/// Built by [`crate::scan`]; consumed by [`Family`] / [`FamilyDef`] to resolve
/// axis definitions.
#[derive(Debug)]
pub struct ComponentRegistry {
    roles: HashMap<String, Vec<ComponentDef>>,
    max_visits: usize,
    /// Per-head override of the visit limit. When a head appears in this map,
    /// the value is used in place of [`Self::max_visits`] during expansion.
    /// Lets one cycle-anchor head (e.g. `HeapExtract`) keep a higher budget
    /// while intermediate types (e.g. `SequentialSet`, `RecursiveQuickSelect`)
    /// stay at `1` so the cycle wraps once instead of multiplying.
    head_max_visits: HashMap<String, usize>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self {
            roles: HashMap::new(),
            max_visits: DEFAULT_MAX_VISITS,
            head_max_visits: HashMap::new(),
        }
    }
}

impl ComponentRegistry {
    /// Return every component registered under `role`, in discovery order.
    /// Returns an empty slice if the role is unknown.
    pub fn role(&self, role: &str) -> &[ComponentDef] {
        self.roles.get(role).map_or(&[], Vec::as_slice)
    }

    /// Default per-path head-visit limit honored by [`expand_role`]. Used for
    /// any head not in [`Self::head_max_visits`]. Default is
    /// [`DEFAULT_MAX_VISITS`].
    pub fn max_visits(&self) -> usize {
        self.max_visits
    }

    /// Override the *default* head-count limit. `n = 1` disables self-
    /// recursion for every head without an explicit override (each head can
    /// be entered at most once on a path, so cycles never close); `n = 2`
    /// allows one self-recursion; larger values nest deeper.
    pub fn set_max_visits(&mut self, n: usize) {
        self.max_visits = n;
    }

    /// Per-head visit budget honored by [`expand_role`]. Returns the override
    /// if present, otherwise [`Self::max_visits`].
    pub fn max_visits_for(&self, head: &str) -> usize {
        self.head_max_visits.get(head).copied().unwrap_or(self.max_visits)
    }

    /// Override the visit budget for one head identifier (the part before
    /// the first `<` of a `type_expr`, see [`type_head`]). Intended for
    /// metadata-declared `max_visits = N` on a composite component.
    pub fn set_head_max_visits(&mut self, head: impl Into<String>, n: usize) {
        self.head_max_visits.insert(head.into(), n);
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

// ── Recursive expansion (head-count rule) ────────────────────────────────────

/// Default per-path head-visit limit. `2` lets a type self-recurse exactly
/// once. Tunable per registry via [`ComponentRegistry::set_max_visits`].
pub const DEFAULT_MAX_VISITS: usize = 2;

/// Head identifier of a type expression — everything before the first `<`,
/// trimmed. `"QuickBuild<{P}>"` → `"QuickBuild"`, `"Lomuto"` → `"Lomuto"`.
/// Used as the type identity in the per-path visit map.
fn type_head(type_expr: &str) -> &str {
    type_expr.split('<').next().unwrap_or(type_expr).trim()
}

/// Expand every component registered under `role` into concrete, slot-free
/// [`ComponentDef`]s, recursively filling composite slots from the registry.
///
/// Recursion is bounded by the **head-count rule**: on any single root→leaf
/// path, each *composite* type head may be entered at most
/// [`ComponentRegistry::max_visits`] times (default [`DEFAULT_MAX_VISITS`] = 2
/// → one self-recursion). Termination follows because every recursive step
/// increments some head's visit count, the catalog of heads is finite, and
/// each count is capped — so the search tree has bounded depth.
///
/// Leaf components (empty `slots`) expand to themselves and don't participate
/// in the limit. A registry with no composite components returns each role's
/// list unchanged.
pub fn expand_role(registry: &ComponentRegistry, role: &str) -> Vec<ComponentDef> {
    let mut out = Vec::new();
    let mut visits: HashMap<String, usize> = HashMap::new();
    for comp in registry.role(role) {
        expand_component(registry, comp, &mut visits, &mut out);
    }
    out
}

/// Expand one component, appending its concrete instantiations to `out`.
/// `visits` tracks how many times each composite head has been entered on the
/// current path (incremented on enter, decremented on exit, so it stays
/// per-path). If the head is already at the registry's limit the component is
/// silently skipped — this is the head-count rule's pruning step.
fn expand_component(
    registry: &ComponentRegistry,
    comp: &ComponentDef,
    visits: &mut HashMap<String, usize>,
    out: &mut Vec<ComponentDef>,
) {
    if comp.slots.is_empty() {
        out.push(comp.clone());
        return;
    }

    let head = type_head(&comp.type_expr).to_string();
    let prior = visits.get(&head).copied().unwrap_or(0);
    if prior >= registry.max_visits_for(&head) {
        return;
    }
    visits.insert(head.clone(), prior + 1);

    // For each slot, the concrete child options legal at this point on the
    // path (head-count-pruned, then recursively expanded to leaves).
    let mut slot_options: Vec<Vec<ComponentDef>> = Vec::with_capacity(comp.slots.len());
    for slot in &comp.slots {
        let mut opts = Vec::new();
        for child in registry.role(&slot.role) {
            expand_component(registry, child, visits, &mut opts);
        }
        slot_options.push(opts);
    }

    // Cartesian product across slots → one concrete ComponentDef per combo,
    // substituting `{param}` in type_expr / label / path_segments and unioning
    // uses. Each `{param}` path segment is *spliced* — replaced by the chosen
    // child's full path_segments — so a composite contributes one segment per
    // recursion level rather than collapsing to a single flat segment.
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
        let mut path_segments: Vec<String> = Vec::new();
        for seg in &comp.path_segments {
            let placeholder = seg.strip_prefix('{').and_then(|s| s.strip_suffix('}'));
            let mut spliced = false;
            if let Some(param) = placeholder {
                if let Some((_, chosen)) =
                    comp.slots.iter().zip(&combo).find(|(slot, _)| slot.param == param)
                {
                    path_segments.extend(chosen.path_segments.iter().cloned());
                    spliced = true;
                }
            }
            if !spliced {
                path_segments.push(seg.clone());
            }
        }
        out.push(ComponentDef { type_expr, label, uses, slots: Vec::new(), path_segments });
    }

    if prior == 0 {
        visits.remove(&head);
    } else {
        visits.insert(head, prior);
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
///
/// The merged component's `uses` is the union of both constituents' `uses`,
/// so identifier-aliasing at emit time has the full lookup context for
/// every name appearing in the combined `type_expr` (e.g. both `L` and `R`
/// in `CombinedSelector<L, R>`).
pub fn cross_axis(
    left: &[ComponentDef],
    right: &[ComponentDef],
    type_fn: impl Fn(&ComponentDef, &ComponentDef) -> String,
    label_fn: impl Fn(&ComponentDef, &ComponentDef) -> String,
) -> Vec<ComponentDef> {
    let mut out = Vec::with_capacity(left.len() * right.len());
    for l in left {
        for r in right {
            let mut uses = l.uses.clone();
            for u in &r.uses {
                if !uses.contains(u) {
                    uses.push(u.clone());
                }
            }
            out.push(ComponentDef::with_uses(type_fn(l, r), label_fn(l, r), uses));
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
    pub fn render(
        &self,
        out: &mut String,
        registry: &ComponentRegistry,
        config: &CodegenConfig,
        alias_map: &HashMap<String, String>,
        module_fallback: &HashMap<String, String>,
    ) {
        self.render_single(out, registry, config, alias_map, module_fallback);
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
        alias_map: &HashMap<String, String>,
        module_fallback: &HashMap<String, String>,
    ) {
        // Resolve every axis upfront so we can inspect its components and
        // decide on the emit strategy (flat vs. per-value explosion).
        let resolved: Vec<(String, AxisSpec, Vec<ComponentDef>)> = self
            .axes
            .iter()
            .map(|(v, s)| (v.clone(), s.clone(), resolve_axis_spec(s, registry)))
            .collect();

        // A "tiered" axis is one whose values include at least one composite
        // — `path_segments.len() > 1`. We explode those axes: one emission per
        // value, with the chosen path_segments spliced into the family's
        // `path` field. Non-tiered axes stay as faceted blocks inside each
        // emission.
        let tiered: Vec<usize> = resolved
            .iter()
            .enumerate()
            .filter(|(_, (_, _, comps))| comps.iter().any(|c| c.path_segments.len() > 1))
            .map(|(i, _)| i)
            .collect();

        if tiered.is_empty() {
            self.emit_one(out, config, &resolved, &[], alias_map, module_fallback);
            return;
        }

        // Cartesian product over the tiered axes — emit one block per combo.
        let tiered_options: Vec<&[ComponentDef]> =
            tiered.iter().map(|&i| resolved[i].2.as_slice()).collect();
        for picks in cartesian_indices(&tiered_options) {
            let bindings: Vec<(usize, &ComponentDef)> = tiered
                .iter()
                .zip(&picks)
                .map(|(&axis_idx, &val_idx)| (axis_idx, &resolved[axis_idx].2[val_idx]))
                .collect();
            self.emit_one(out, config, &resolved, &bindings, alias_map, module_fallback);
        }
    }

    /// Emit one `<output_macro>! { … }` block.
    ///
    /// `bindings` is the per-emission pre-substituted slice of tiered axes:
    /// each entry is `(axis index in resolved, chosen component)`. For those
    /// axes:
    ///
    /// - The `{var}` placeholder in `type_template` is replaced by the
    ///   component's `type_expr`.
    /// - The `{var}` placeholder in the `path` field is *spliced* with the
    ///   component's full `path_segments`, turning one segment into N.
    /// - The axis is omitted from the macro's `axes` block (it's already
    ///   resolved).
    /// - The family `name` field is suffixed with the tiered labels to keep
    ///   Rust identifiers unique across emissions (the consumer macro derives
    ///   identifiers from `name`).
    ///
    /// Non-tiered axes render as faceted blocks just like the pre-explosion
    /// behavior.
    fn emit_one(
        &self,
        out: &mut String,
        config: &CodegenConfig,
        resolved: &[(String, AxisSpec, Vec<ComponentDef>)],
        bindings: &[(usize, &ComponentDef)],
        alias_map: &HashMap<String, String>,
        module_fallback: &HashMap<String, String>,
    ) {
        use std::fmt::Write as _;

        writeln!(out, "{}! {{", config.output_macro).unwrap();

        // Per-emission alias lookup order:
        // 1. The component's own `uses` (always wins when present — disambiguates collisions).
        // 2. The family's `uses` (e.g. inline-axis types listed only at the family level).
        // 3. The module-level fallback (unambiguous short names from any
        //    component/family in this module — catches the inline-axis case
        //    where neither component nor family lists every identifier).
        let family_local = local_alias_map(&self.uses, alias_map);
        let alias_for_component = |comp: &ComponentDef| -> HashMap<String, String> {
            let mut m = local_alias_map(&comp.uses, alias_map);
            for (k, v) in &family_local {
                m.entry(k.clone()).or_insert_with(|| v.clone());
            }
            for (k, v) in module_fallback {
                m.entry(k.clone()).or_insert_with(|| v.clone());
            }
            m
        };

        // Substitute tiered placeholders in the type template, alias-rewriting
        // each component's type_expr first so the final string carries the
        // collision-safe `<short>_<N>` identifiers.
        let mut type_template = self.type_template.clone();
        for (axis_idx, comp) in bindings {
            let var = &resolved[*axis_idx].0;
            let ph = format!("{{{}}}", var);
            let comp_aliased = substitute_aliases(&comp.type_expr, &alias_for_component(comp));
            type_template = type_template.replace(&ph, &comp_aliased);
        }
        // Now alias the family-owned identifiers (e.g. `QuickSort`, `NoPivot`).
        // Already-aliased component identifiers like `LeftLeftPartition_0` are
        // single tokens and won't match a bare `LeftLeftPartition` in the
        // family's local map, so double-aliasing can't happen. The module
        // fallback fills in identifiers neither the component nor the
        // family enumerated (inline-axis nested types).
        let mut tpl_map = family_local.clone();
        for (k, v) in module_fallback {
            tpl_map.entry(k.clone()).or_insert_with(|| v.clone());
        }
        let type_template = substitute_aliases(&type_template, &tpl_map);
        writeln!(out, "    {}{};", config.type_prefix, type_template).unwrap();
        out.push('\n');

        // Emit axes block for non-tiered axes only.
        let bound: Vec<usize> = bindings.iter().map(|(i, _)| *i).collect();
        for (i, (var, spec, components)) in resolved.iter().enumerate() {
            if bound.contains(&i) {
                continue;
            }
            let role = Self::axis_role(spec);
            if role.is_empty() {
                writeln!(out, "    {var} {{").unwrap();
            } else {
                writeln!(out, "    {var} : \"{role}\" {{").unwrap();
            }
            for comp in components {
                let comp_aliased =
                    substitute_aliases(&comp.type_expr, &alias_for_component(comp));
                writeln!(out, "        {} => \"{}\"", comp_aliased, comp.label).unwrap();
            }
            out.push_str("    }\n");
        }

        out.push('\n');

        let key_width = self.fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

        for (key, value) in &self.fields {
            let rendered = self.render_field(key, value, config, resolved, bindings);
            writeln!(out, "    {key:<key_width$} = {};", rendered).unwrap();
        }

        out.push_str("}\n\n");
    }

    /// Per-emission field rendering. Two fields are special-cased:
    ///
    /// - The path field (named by [`CodegenConfig::path_field`]): each tiered
    ///   `{var}` is spliced with the chosen component's path_segments.
    /// - The `name` field: when there are tiered bindings, the chosen labels
    ///   are joined and appended, making the per-emission name unique. The
    ///   downstream `sort_family!` macro derives Rust identifiers from this
    ///   name, so this is what keeps 40 explosions of the same family from
    ///   colliding on `__sf_<family>_<idx>_<labels>` identifiers.
    fn render_field(
        &self,
        key: &str,
        value: &FieldValue,
        config: &CodegenConfig,
        resolved: &[(String, AxisSpec, Vec<ComponentDef>)],
        bindings: &[(usize, &ComponentDef)],
    ) -> String {
        if Some(key) == config.path_field.as_deref() {
            if let FieldValue::StringArray(segs) = value {
                let expanded = expand_path_segments(segs, resolved, bindings);
                let inner = expanded
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                return format!("[{inner}]");
            }
        }
        if key == "name" && !bindings.is_empty() {
            if let FieldValue::String(base) = value {
                let suffix = bindings
                    .iter()
                    .map(|(_, c)| c.label.as_str())
                    .collect::<Vec<_>>()
                    .join(" / ");
                return format!("\"{base}: {suffix}\"");
            }
        }
        value.render()
    }
}

/// Substitute each `{var}` segment in `template` with the bound axis's
/// `path_segments`. Untouched segments (literals and non-tiered placeholders)
/// pass through verbatim — the consumer macro substitutes non-tiered
/// placeholders on its own from the variant's leaf labels.
fn expand_path_segments(
    template: &[String],
    resolved: &[(String, AxisSpec, Vec<ComponentDef>)],
    bindings: &[(usize, &ComponentDef)],
) -> Vec<String> {
    let mut out = Vec::new();
    for seg in template {
        let placeholder = seg.strip_prefix('{').and_then(|s| s.strip_suffix('}'));
        let mut spliced = false;
        if let Some(var) = placeholder {
            let bound_comp = resolved
                .iter()
                .enumerate()
                .find(|(_, (v, _, _))| v == var)
                .and_then(|(i, _)| bindings.iter().find(|(j, _)| *j == i).map(|(_, c)| *c));
            if let Some(comp) = bound_comp {
                out.extend(comp.path_segments.iter().cloned());
                spliced = true;
            }
        }
        if !spliced {
            out.push(seg.clone());
        }
    }
    out
}

/// Cartesian product of per-axis component index spaces. Each output is a
/// `Vec<usize>` of the same length as `options`, picking one index from each.
fn cartesian_indices(options: &[&[ComponentDef]]) -> Vec<Vec<usize>> {
    let mut combos: Vec<Vec<usize>> = vec![Vec::new()];
    for opts in options {
        let mut next = Vec::with_capacity(combos.len() * opts.len());
        for combo in &combos {
            for i in 0..opts.len() {
                let mut extended = combo.clone();
                extended.push(i);
                next.push(extended);
            }
        }
        combos = next;
    }
    combos
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

    // ── Recursive expansion / head-count rule ──────────────────────────────────

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
    /// Without the head-count rule this loops forever
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
    fn expand_role_head_count_default_terminates_with_expected_set() {
        let reg = recursive_registry();
        assert_eq!(reg.max_visits(), DEFAULT_MAX_VISITS);
        let out = expand_role(&reg, "Partition");
        let types: Vec<&str> = out.iter().map(|c| c.type_expr.as_str()).collect();
        // With max_visits = 2, each composite head can be entered at most
        // twice on a path. HeapExtract appears at the root and once inside,
        // then can't appear again; same for QuickBuild. The deepest variants
        // bottom out in Lomuto/Hoare leaves once both heads are at their cap.
        assert_eq!(
            types,
            vec![
                "Lomuto",
                "Hoare",
                "HeapExtract<SimpleBuild>",
                "HeapExtract<QuickBuild<Lomuto>>",
                "HeapExtract<QuickBuild<Hoare>>",
                "HeapExtract<QuickBuild<HeapExtract<SimpleBuild>>>",
                "HeapExtract<QuickBuild<HeapExtract<QuickBuild<Lomuto>>>>",
                "HeapExtract<QuickBuild<HeapExtract<QuickBuild<Hoare>>>>",
            ],
        );
    }

    #[test]
    fn expand_role_respects_head_visit_cap() {
        let reg = recursive_registry();
        let types: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.type_expr).collect();
        // No type appears more than max_visits = 2 times on any path.
        for t in &types {
            assert!(
                t.matches("HeapExtract").count() <= reg.max_visits(),
                "HeapExtract exceeded max_visits in {t}",
            );
            assert!(
                t.matches("QuickBuild").count() <= reg.max_visits(),
                "QuickBuild exceeded max_visits in {t}",
            );
        }
        // Labels are templated too.
        let labels: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.label).collect();
        assert!(labels.contains(&"heap extract<quick build<lomuto>>".to_string()));
    }

    #[test]
    fn expand_role_max_visits_one_disables_recursion() {
        let mut reg = recursive_registry();
        reg.set_max_visits(1);
        let out = expand_role(&reg, "Partition");
        let types: Vec<&str> = out.iter().map(|c| c.type_expr.as_str()).collect();
        // With max_visits = 1, no head can self-recurse: HeapExtract appears
        // only at the root, QuickBuild can fill its slot but cannot wrap
        // another HeapExtract.
        assert_eq!(
            types,
            vec![
                "Lomuto",
                "Hoare",
                "HeapExtract<SimpleBuild>",
                "HeapExtract<QuickBuild<Lomuto>>",
                "HeapExtract<QuickBuild<Hoare>>",
            ],
        );
    }

    #[test]
    fn expand_role_per_head_limit_shrinks_intermediates() {
        // Default global = 2 but cap QuickBuild to 1. HeapExtract still gets
        // its full budget, so the cycle closes once via HeapExtract<...>, but
        // QuickBuild — used as the intermediate — can't reappear at depth 2.
        let mut reg = recursive_registry();
        reg.set_head_max_visits("QuickBuild", 1);
        let types: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.type_expr).collect();
        assert_eq!(
            types,
            vec![
                "Lomuto".to_string(),
                "Hoare".to_string(),
                "HeapExtract<SimpleBuild>".to_string(),
                "HeapExtract<QuickBuild<Lomuto>>".to_string(),
                "HeapExtract<QuickBuild<Hoare>>".to_string(),
                "HeapExtract<QuickBuild<HeapExtract<SimpleBuild>>>".to_string(),
            ],
        );
        // No occurrence of QuickBuild twice on any path.
        for t in &types {
            assert!(
                t.matches("QuickBuild").count() <= 1,
                "QuickBuild exceeded its per-head cap of 1 in {t}",
            );
        }
    }

    #[test]
    fn expand_role_max_visits_three_allows_deeper_nesting() {
        let mut reg = recursive_registry();
        reg.set_max_visits(3);
        let types: Vec<String> =
            expand_role(&reg, "Partition").into_iter().map(|c| c.type_expr).collect();
        // With max_visits = 3, each head may appear up to 3 times, so we get
        // strictly more variants than the default. Sanity-check via the
        // deepest expected shape.
        assert!(
            types.iter().any(|t| t.matches("HeapExtract").count() == 3),
            "expected a triple-nested HeapExtract at max_visits=3: {types:?}",
        );
        for t in &types {
            assert!(t.matches("HeapExtract").count() <= 3);
            assert!(t.matches("QuickBuild").count() <= 3);
        }
    }

    #[test]
    fn alias_map_disambiguates_short_name_collision() {
        let paths = vec![
            "heap_sort_lib::heap_partition::LeftLeftPartition".to_string(),
            "partition_lomuto::LeftLeftPartition".to_string(),
            "heap_sort_lib::arity::Binary".to_string(),
        ];
        let map = build_alias_map(&paths);
        // The two LeftLeftPartition paths must end up with distinct aliases —
        // that's the whole point.
        let a = map.get("heap_sort_lib::heap_partition::LeftLeftPartition").unwrap();
        let b = map.get("partition_lomuto::LeftLeftPartition").unwrap();
        assert_ne!(a, b);
        assert!(a.starts_with("LeftLeftPartition_"));
        assert!(b.starts_with("LeftLeftPartition_"));
        // Singletons still take a `_<N>` suffix (uniform shape).
        let bin = map.get("heap_sort_lib::arity::Binary").unwrap();
        assert!(bin.starts_with("Binary_"));
    }

    #[test]
    fn substitute_aliases_respects_identifier_boundaries() {
        let mut map = HashMap::new();
        map.insert("HeapExtract".to_string(), "HeapExtract_3".to_string());
        map.insert("Binary".to_string(), "Binary_0".to_string());
        // `HeapExtract` and `Binary` get rewritten as whole tokens; the `<`,
        // `,`, ` ` are boundaries; `BinaryFoo` (no map entry) must NOT be
        // partially rewritten to `Binary_0Foo`.
        let out = substitute_aliases("HeapExtract<Binary, BinaryFoo>", &map);
        assert_eq!(out, "HeapExtract_3<Binary_0, BinaryFoo>");
    }

    #[test]
    fn expand_grouped_uses_splits_brace_groups() {
        let uses = vec![
            "heap_sort_lib::heap_sort::{HeapSort, NaryHeapSort}".to_string(),
            "plain::Path".to_string(),
        ];
        let out = expand_grouped_uses(&uses);
        assert_eq!(
            out,
            vec![
                "heap_sort_lib::heap_sort::HeapSort".to_string(),
                "heap_sort_lib::heap_sort::NaryHeapSort".to_string(),
                "plain::Path".to_string(),
            ],
        );
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
