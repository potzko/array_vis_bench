//! Component discovery via Cargo.toml `[package.metadata.array_vis_bench]`.
//!
//! Replaces the legacy text-grep scanner in [`crate::scanner`]. Each
//! component is one entry in a TOML array-of-tables:
//!
//! ```toml
//! [[package.metadata.array_vis_bench.components]]
//! role  = "Partition"
//! type  = "LeftLeftPartition"
//! label = "left-left pointer"
//! ```
//!
//! Array form is preferred over a keyed map because the same `type` can
//! appear under multiple roles (e.g. `InsertionSmallSort<LinearInsertion,
//! 16>` is registered as both `SmallSort` and `NonTrivialSmallSort`), and
//! TOML map keys must be unique within a parent table.
//!
//! Phase 1 reads the current crate's `Cargo.toml` only; the dep-graph
//! walk needed when components live in sibling crates is a later phase.
//! Reserved fields (`generic_axes`, `label_template`) are accepted but
//! not yet consumed.
//!
//! ### Validation
//!
//! Strict by design — unknown fields are rejected so a typo can't silently
//! drop a component. The build script wraps the parse error in its own
//! panic that points at the offending manifest path.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::family::{AxisSpec, ComponentDef, FamilyDef, FieldValue, Slot};

/// One component discovered from a Cargo.toml metadata block.
#[derive(Debug, Clone)]
pub struct MetadataComponent {
    pub role: String,
    pub type_expr: String,
    pub label: String,
    /// `use` paths this component needs in the generated file.
    pub uses: Vec<String>,
    /// Recursive parameter slots (`{ param, role }`). Empty for leaf
    /// components; non-empty makes the component generic over a role, expanded
    /// by [`crate::expand_role`] under the head-count rule.
    pub slots: Vec<Slot>,
    /// Optional per-head visit cap. When set, the build script feeds this
    /// into [`crate::ComponentRegistry::set_head_max_visits`] so this
    /// component's head can have a different recursion budget than the
    /// registry default. Used to keep intermediate cycle-participant heads
    /// at `1` while one anchor head stays at the default.
    pub max_visits: Option<usize>,
    /// Path to the `Cargo.toml` it came from — used for `cargo:rerun-if-changed`.
    pub source_manifest: PathBuf,
}

/// One family discovered from a Cargo.toml metadata block. Distinct from
/// [`FamilyDef`] only by carrying the manifest path so the build script
/// can emit a `rerun-if-changed` line for each family-bearing crate.
#[derive(Debug, Clone)]
pub struct MetadataFamily {
    pub family: FamilyDef,
    pub source_manifest: PathBuf,
}

/// Errors specific to metadata discovery. Read errors are surfaced as the
/// underlying `io::Error`; parse errors include the manifest path so the
/// caller can point a build-script panic at the right file.
#[derive(Debug)]
pub enum MetadataError {
    Io(std::io::Error),
    Parse {
        manifest: PathBuf,
        source: toml::de::Error,
    },
    /// `cargo metadata` returns the metadata block as a `serde_json::Value`
    /// (Cargo serializes TOML metadata through JSON internally), so the
    /// dep-graph path deserializes from JSON and surfaces JSON errors
    /// separately from the single-manifest TOML path.
    JsonParse {
        manifest: PathBuf,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::Io(e) => write!(f, "{e}"),
            MetadataError::Parse { manifest, source } => {
                write!(f, "{}: {source}", manifest.display())
            }
            MetadataError::JsonParse { manifest, source } => {
                write!(f, "{}: {source}", manifest.display())
            }
        }
    }
}

impl std::error::Error for MetadataError {}

impl From<std::io::Error> for MetadataError {
    fn from(e: std::io::Error) -> Self {
        MetadataError::Io(e)
    }
}

/// Walk the workspace + dependency graph rooted at `manifest_path` and
/// return every component declared across all reachable crates'
/// `[package.metadata.array_vis_bench.components]` blocks. Order is:
/// stable per crate (TOML declaration order), then crates in
/// topological-ish order from `cargo_metadata`'s `packages` list.
///
/// This is the discovery primitive that makes per-leaf crates work —
/// a wiring crate's `build.rs` calls this once and the scanner finds
/// component declarations in all transitive deps without each level
/// having to forward metadata.
pub fn scan_workspace_components(
    manifest_path: impl AsRef<Path>,
) -> Result<Vec<MetadataComponent>, MetadataError> {
    let manifest_path = manifest_path.as_ref();
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .map_err(|e| MetadataError::Io(std::io::Error::other(format!("cargo metadata: {e}"))))?;

    let mut out = Vec::new();
    for pkg in &metadata.packages {
        let avb = match pkg
            .metadata
            .get("array_vis_bench")
            .and_then(|m| m.get("components"))
        {
            Some(c) => c,
            None => continue,
        };
        // `cargo_metadata` returns the metadata field as `serde_json::Value`
        // (Cargo serializes Cargo.toml's TOML metadata block through JSON).
        // Round-trip through serde_json into our existing TOML-shaped
        // schema — the field types are identical so this is cheap and
        // keeps one parse path for `deny_unknown_fields` enforcement.
        let decls: Vec<ComponentDecl> =
            serde_json::from_value(avb.clone()).map_err(|source| MetadataError::JsonParse {
                manifest: pkg.manifest_path.clone().into_std_path_buf(),
                source,
            })?;
        for decl in decls {
            // If the component doesn't declare its own `uses`, auto-derive a
            // single import from its source crate + the base type name (the
            // type stripped of any generic args). This covers the common case
            // — a leaf crate exposing one type at its root — with no metadata.
            // Components whose type lives in another crate (e.g. the
            // `*_components` metadata-only crates) or that carry generic-arg
            // types must list `uses` explicitly.
            let uses = if decl.uses.is_empty() {
                let base = decl.type_expr.split('<').next().unwrap_or(&decl.type_expr).trim();
                let krate = pkg.name.as_str().replace('-', "_");
                vec![format!("{krate}::{base}")]
            } else {
                decl.uses
            };
            out.push(MetadataComponent {
                role: decl.role,
                type_expr: decl.type_expr,
                label: decl.label,
                uses,
                slots: decl.slots.into_iter().map(SlotDecl::into_slot).collect(),
                max_visits: decl.max_visits,
                source_manifest: pkg.manifest_path.clone().into_std_path_buf(),
            });
        }
    }
    Ok(out)
}

/// Walk the workspace + dependency graph rooted at `manifest_path` and
/// return every family declared across all reachable crates'
/// `[[package.metadata.array_vis_bench.families]]` blocks. The
/// family-axis-flavoured analogue of [`scan_workspace_components`].
///
/// Each family's `module` field becomes its [`FamilyDef::source_module`]
/// — the build script groups generated output by that key, so two
/// families with `module = "quick_sorts"` (in different leaf crates)
/// emit into the same `quick_sorts_combinations.rs`. If the field is
/// missing, the crate name is used.
pub fn scan_workspace_families(
    manifest_path: impl AsRef<Path>,
) -> Result<Vec<MetadataFamily>, MetadataError> {
    let manifest_path = manifest_path.as_ref();
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .map_err(|e| MetadataError::Io(std::io::Error::other(format!("cargo metadata: {e}"))))?;

    let mut out = Vec::new();
    for pkg in &metadata.packages {
        let avb = match pkg
            .metadata
            .get("array_vis_bench")
            .and_then(|m| m.get("families"))
        {
            Some(c) => c,
            None => continue,
        };
        let arr = avb.as_array().ok_or_else(|| MetadataError::JsonParse {
            manifest: pkg.manifest_path.clone().into_std_path_buf(),
            source: serde::de::Error::custom("`families` must be an array"),
        })?;
        for raw in arr {
            let family = parse_family_value(
                raw,
                pkg.name.as_str(),
                &pkg.manifest_path.clone().into_std_path_buf(),
            )?;
            out.push(MetadataFamily {
                family,
                source_manifest: pkg.manifest_path.clone().into_std_path_buf(),
            });
        }
    }
    Ok(out)
}

/// Read a single `Cargo.toml` and return every component declared in its
/// `[package.metadata.array_vis_bench.components.*]` blocks. Returns an
/// empty `Vec` if the file has no such block — that is the normal case
/// for crates that aren't components themselves.
///
/// Prefer [`scan_workspace_components`] when the wiring crate has
/// component-bearing dep crates — `scan_manifest` only reads the single
/// `Cargo.toml` it's pointed at.
pub fn scan_manifest(manifest: impl AsRef<Path>) -> Result<Vec<MetadataComponent>, MetadataError> {
    let path = manifest.as_ref();
    let text = std::fs::read_to_string(path)?;
    let parsed: Manifest = toml::from_str(&text).map_err(|source| MetadataError::Parse {
        manifest: path.to_path_buf(),
        source,
    })?;

    let components = parsed
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.array_vis_bench)
        .map(|a| a.components)
        .unwrap_or_default();

    let out: Vec<MetadataComponent> = components
        .into_iter()
        .map(|decl| MetadataComponent {
            role: decl.role,
            type_expr: decl.type_expr,
            label: decl.label,
            uses: decl.uses,
            slots: decl.slots.into_iter().map(SlotDecl::into_slot).collect(),
            max_visits: decl.max_visits,
            source_manifest: path.to_path_buf(),
        })
        .collect();
    Ok(out)
}

// ── Family parser ────────────────────────────────────────────────────────────
//
// Family declarations live in `[[package.metadata.array_vis_bench.families]]`.
// Each entry is structurally similar to the legacy `family!(…)` macro
// call, with these slots:
//
//   module = "quick_sorts"          # generated file basename (optional;
//                                   # defaults to crate name)
//   type   = "QuickSort<{P}, {V}>"  # generic type template
//   uses   = ["crate::…", …]        # `use` paths in the generated file
//   axes   = [                      # ordered list of (var, spec)
//     { var = "P", role = "Partition" },
//     { var = "V", role = "PivotSelector" },
//     # cross + extras:
//     # { var = "DPS",
//     #   cross = { left = "PivotSelector", right = "PivotSelector",
//     #            type_tmpl = "Combined<{0}, {1}>", label_tmpl = "{0}/{1}" },
//     #   extras = [{ type = "Ninther", label = "ninther" }] },
//     # inline:
//     # { var = "PP", inline = [{ type = "false", label = "" }, …] },
//   ]
//
// Every other key/value pair at the top level is a trailing field, in
// TOML declaration order — these end up on the right-hand side of the
// generated `name = "…"; big_o = inherited; …` block.
//
// The pass-through `inherited` keyword (a bare ident in macro syntax)
// is spelled `"@inherited"` in TOML — anything starting with `@` becomes
// a [`FieldValue::Ident`]. Without the sentinel, `inherited` would
// collide with a regular string literal.

fn parse_family_value(
    value: &serde_json::Value,
    crate_name: &str,
    manifest: &Path,
) -> Result<FamilyDef, MetadataError> {
    let table = value.as_object().ok_or_else(|| MetadataError::JsonParse {
        manifest: manifest.to_path_buf(),
        source: serde::de::Error::custom("family entry must be a table"),
    })?;

    let mut type_template: Option<String> = None;
    let mut uses: Vec<String> = Vec::new();
    let mut axes: Vec<(String, AxisSpec)> = Vec::new();
    let mut module: Option<String> = None;
    let mut fields: Vec<(String, FieldValue)> = Vec::new();

    for (key, val) in table {
        match key.as_str() {
            "module" => {
                module = Some(val.as_str().ok_or_else(|| field_err(manifest, "module must be a string"))?.to_string());
            }
            "type" => {
                type_template = Some(val.as_str().ok_or_else(|| field_err(manifest, "type must be a string"))?.to_string());
            }
            "uses" => {
                uses = json_to_string_array(val).ok_or_else(|| field_err(manifest, "uses must be an array of strings"))?;
            }
            "axes" => {
                let arr = val.as_array().ok_or_else(|| field_err(manifest, "axes must be an array of tables"))?;
                for ax in arr {
                    axes.push(parse_axis_value(ax, manifest)?);
                }
            }
            _ => {
                let fv = json_to_field_value(val).ok_or_else(|| {
                    field_err(manifest, &format!("field `{key}` has an unsupported value shape"))
                })?;
                fields.push((key.clone(), fv));
            }
        }
    }

    let type_template = type_template
        .ok_or_else(|| field_err(manifest, "family is missing required `type` field"))?;
    let source_module = module.unwrap_or_else(|| crate_name.to_string());

    Ok(FamilyDef { type_template, axes, uses, fields, source_module })
}

fn parse_axis_value(
    value: &serde_json::Value,
    manifest: &Path,
) -> Result<(String, AxisSpec), MetadataError> {
    let table = value.as_object().ok_or_else(|| field_err(manifest, "axis must be a table"))?;
    let var = table
        .get("var")
        .and_then(|v| v.as_str())
        .ok_or_else(|| field_err(manifest, "axis is missing `var`"))?
        .to_string();

    if let Some(role) = table.get("role") {
        let role = role
            .as_str()
            .ok_or_else(|| field_err(manifest, "axis.role must be a string"))?
            .to_string();
        return Ok((var, AxisSpec::Role(role)));
    }
    if let Some(cross) = table.get("cross") {
        let cross_tbl = cross
            .as_object()
            .ok_or_else(|| field_err(manifest, "axis.cross must be a table"))?;
        let left = cross_tbl
            .get("left")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "axis.cross.left missing"))?
            .to_string();
        let right = cross_tbl
            .get("right")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "axis.cross.right missing"))?
            .to_string();
        let type_tmpl = cross_tbl
            .get("type_tmpl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "axis.cross.type_tmpl missing"))?
            .to_string();
        let label_tmpl = cross_tbl
            .get("label_tmpl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "axis.cross.label_tmpl missing"))?
            .to_string();
        let extras = match table.get("extras") {
            Some(e) => parse_inline_pairs(e, manifest)?,
            None => Vec::new(),
        };
        return Ok((var, AxisSpec::Cross { left, right, type_tmpl, label_tmpl, extras }));
    }
    if let Some(inline) = table.get("inline") {
        let items = parse_inline_pairs(inline, manifest)?;
        return Ok((var, AxisSpec::Inline(items)));
    }
    Err(field_err(manifest, "axis must have one of `role`, `cross`, or `inline`"))
}

fn parse_inline_pairs(
    value: &serde_json::Value,
    manifest: &Path,
) -> Result<Vec<ComponentDef>, MetadataError> {
    let arr = value.as_array().ok_or_else(|| field_err(manifest, "inline / extras must be an array"))?;
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        let tbl = v.as_object().ok_or_else(|| field_err(manifest, "inline entry must be a table"))?;
        let ty = tbl
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "inline entry missing `type`"))?
            .to_string();
        let label = tbl
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| field_err(manifest, "inline entry missing `label`"))?
            .to_string();
        out.push(ComponentDef::new(ty, label));
    }
    Ok(out)
}

fn json_to_string_array(value: &serde_json::Value) -> Option<Vec<String>> {
    let arr = value.as_array()?;
    arr.iter().map(|v| v.as_str().map(String::from)).collect()
}

fn json_to_field_value(value: &serde_json::Value) -> Option<FieldValue> {
    use serde_json::Value as V;
    match value {
        V::Bool(b) => Some(FieldValue::Bool(*b)),
        V::Number(n) => n.as_i64().map(FieldValue::Int),
        V::String(s) => {
            if let Some(ident) = s.strip_prefix('@') {
                Some(FieldValue::Ident(ident.to_string()))
            } else {
                Some(FieldValue::String(s.clone()))
            }
        }
        V::Array(arr) => arr
            .iter()
            .map(|v| v.as_str().map(String::from))
            .collect::<Option<Vec<_>>>()
            .map(FieldValue::StringArray),
        _ => None,
    }
}

fn field_err(manifest: &Path, msg: &str) -> MetadataError {
    MetadataError::JsonParse {
        manifest: manifest.to_path_buf(),
        source: <serde_json::Error as serde::de::Error>::custom(msg),
    }
}

// ── TOML schema ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Manifest {
    package: Option<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: Option<MetadataRoot>,
}

#[derive(Debug, Deserialize)]
struct MetadataRoot {
    array_vis_bench: Option<AvbMetadata>,
}

#[derive(Debug, Deserialize, Default)]
struct AvbMetadata {
    /// Vec preserves declaration order. Array-of-tables (`[[components]]`)
    /// in TOML maps naturally onto this.
    #[serde(default)]
    components: Vec<ComponentDecl>,
}

/// `deny_unknown_fields` is the load-bearing strictness check — a typo'd
/// key (e.g. `lable` instead of `label`) errors at build time instead of
/// silently producing a component with the wrong label.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ComponentDecl {
    role: String,
    #[serde(rename = "type")]
    type_expr: String,
    label: String,
    /// `use` paths this component needs in the generated file — its own type
    /// path plus any generic-argument types. The emitter unions these into
    /// every family that references this component's role, so families no
    /// longer have to list component imports.
    #[serde(default)]
    uses: Vec<String>,
    /// Recursive parameter slots making this component generic over a role.
    /// Each `{ param, role }` binds a `{param}` placeholder in `type` /
    /// `label` to the components of `role`, expanded recursively under the
    /// head-count rule. Empty (the default) means a leaf component.
    #[serde(default)]
    slots: Vec<SlotDecl>,
    /// Optional per-head visit cap. Overrides the registry default when set,
    /// keyed by this component's head (`type_head(type_expr)`). Lets the
    /// metadata pin one anchor head at the default budget and shrink the
    /// rest to `1` so a cycle wraps once instead of multiplying.
    #[serde(default)]
    max_visits: Option<usize>,
}

/// One recursive slot on a [`ComponentDecl`]. `param` is the `{param}`
/// placeholder; `role` is the registry role whose components fill it.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct SlotDecl {
    param: String,
    role: String,
}

impl SlotDecl {
    fn into_slot(self) -> Slot {
        Slot::new(self.param, self.role)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_manifest(text: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(text.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parses_single_component() {
        let f = write_manifest(
            r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [[package.metadata.array_vis_bench.components]]
            role  = "Partition"
            type  = "LeftLeftPartition"
            label = "left-left pointer"
            "#,
        );
        let out = scan_manifest(f.path()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].role, "Partition");
        assert_eq!(out[0].type_expr, "LeftLeftPartition");
        assert_eq!(out[0].label, "left-left pointer");
    }

    #[test]
    fn parses_recursive_slots() {
        let f = write_manifest(
            r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [[package.metadata.array_vis_bench.components]]
            role  = "Partition"
            type  = "MovingPivotV3<{R}>"
            label = "moving pivot v3<{R}>"
            slots = [{ param = "R", role = "Rotation" }]
            "#,
        );
        let out = scan_manifest(f.path()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].type_expr, "MovingPivotV3<{R}>");
        assert_eq!(out[0].slots.len(), 1);
        assert_eq!(out[0].slots[0].param, "R");
        assert_eq!(out[0].slots[0].role, "Rotation");
    }

    #[test]
    fn no_block_returns_empty() {
        let f = write_manifest(r#"[package]
name = "demo"
version = "0.1.0""#);
        assert!(scan_manifest(f.path()).unwrap().is_empty());
    }

    #[test]
    fn parses_multiple_components_in_declaration_order() {
        let f = write_manifest(
            r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [[package.metadata.array_vis_bench.components]]
            role  = "Partition"
            type  = "LeftLeftPartition"
            label = "left-left pointer"

            [[package.metadata.array_vis_bench.components]]
            role  = "Partition"
            type  = "LeftRightPartition"
            label = "left-right pointer"
            "#,
        );
        let out = scan_manifest(f.path()).unwrap();
        assert_eq!(out.len(), 2);
        // TOML array order is preserved.
        assert_eq!(out[0].label, "left-left pointer");
        assert_eq!(out[1].label, "left-right pointer");
    }

    #[test]
    fn parse_error_reports_manifest_path() {
        let f = write_manifest("this is not valid toml [[[");
        let err = scan_manifest(f.path()).unwrap_err();
        match err {
            MetadataError::Parse { manifest, .. } => {
                assert_eq!(manifest, f.path());
            }
            MetadataError::Io(_) | MetadataError::JsonParse { .. } => {
                panic!("expected Parse error")
            }
        }
    }

    #[test]
    fn unknown_field_errors_at_parse() {
        // `deny_unknown_fields` catches a typo'd key (`lable` instead of
        // `label`) at scan time rather than silently producing a component
        // with no label.
        let f = write_manifest(
            r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [[package.metadata.array_vis_bench.components]]
            role  = "Partition"
            type  = "LeftLeftPartition"
            lable = "left-left pointer"
            "#,
        );
        let err = scan_manifest(f.path()).unwrap_err();
        assert!(matches!(err, MetadataError::Parse { .. }));
    }
}
