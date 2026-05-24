use std::path::{Path, PathBuf};

use crate::family::{AxisSpec, CodegenConfig, ComponentDef, ComponentRegistry, FamilyDef, FieldValue};

/// A single `component!(Role, TypeExpr, "label")` call found in a source file.
#[derive(Debug, Clone)]
struct ScannedComponent {
    role: String,
    type_expr: String,
    label: String,
}

/// Returned by [`scan`]. Holds the discovered registry, scanned families, the
/// config used for scanning (reused at emit time), and the list of files that
/// were read so the caller can emit `cargo:rerun-if-changed` lines.
pub struct ScanResult {
    pub registry: ComponentRegistry,
    pub families: Vec<FamilyDef>,
    pub config: CodegenConfig,
    scanned_files: Vec<PathBuf>,
}

impl ScanResult {
    /// Print a `cargo:rerun-if-changed=<path>` line for every scanned `.rs`
    /// file. Call this from `build.rs` so Cargo reruns the build script
    /// whenever any annotated source file changes.
    pub fn emit_rerun(&self) {
        for path in &self.scanned_files {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    /// Generate one `<module><config.filename_suffix>` file per source module
    /// into `out_dir`. Families are grouped by their
    /// [`FamilyDef::source_module`]. Within each group, `use` declarations are
    /// deduplicated (first-occurrence order) and the resolved
    /// `<config.output_macro>! { … }` blocks are appended.
    pub fn emit_families(&self, out_dir: &Path) -> Result<(), std::io::Error> {
        let mut modules: Vec<&str> = Vec::new();
        for fam in &self.families {
            let m = fam.source_module.as_str();
            if !modules.contains(&m) {
                modules.push(m);
            }
        }

        for module in modules {
            let fams: Vec<&FamilyDef> = self
                .families
                .iter()
                .filter(|f| f.source_module == module)
                .collect();

            let mut out = String::new();

            let mut seen_uses: Vec<&str> = Vec::new();
            for fam in &fams {
                for u in &fam.uses {
                    let u = u.as_str();
                    if !seen_uses.contains(&u) {
                        seen_uses.push(u);
                        out.push_str("use ");
                        out.push_str(u);
                        out.push_str(";\n");
                    }
                }
            }
            out.push('\n');

            for fam in &fams {
                fam.render(&mut out, &self.registry, &self.config);
            }

            let filename = format!("{}{}", module, self.config.filename_suffix);
            std::fs::write(out_dir.join(&filename), &out)?;
        }

        Ok(())
    }
}

/// Recursively walk `dir`, parse every `.rs` file for `component!(...)` and
/// `<config.marker>!(...)` calls, and return the aggregated [`ScanResult`].
///
/// Files are visited in lexicographic path order so the scan — and every
/// downstream piece of generated code — is reproducible across machines and
/// filesystems.
pub fn scan(dir: impl AsRef<Path>, config: &CodegenConfig) -> Result<ScanResult, std::io::Error> {
    let mut registry = ComponentRegistry::default();
    let mut families = Vec::new();
    let mut scanned_files = Vec::new();

    let mut paths: Vec<PathBuf> = walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();
    paths.sort();

    let marker = format!("{}!(", config.marker);

    for path in paths {
        let content = std::fs::read_to_string(&path)?;
        for comp in scan_components(&content) {
            registry.add(comp.role, comp.type_expr, comp.label);
        }
        families.extend(scan_families(&content, &path, &marker));
        scanned_files.push(path);
    }

    Ok(ScanResult { registry, families, config: config.clone(), scanned_files })
}

// ── component! scanner ───────────────────────────────────────────────────────

/// Find all `component!(...)` calls in `content` and return parsed components.
fn scan_components(content: &str) -> Vec<ScannedComponent> {
    const MARKER: &str = "component!(";
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(rel) = content[search_from..].find(MARKER) {
        let args_start = search_from + rel + MARKER.len();
        search_from = args_start;
        if let Some(comp) = parse_component_call(&content[args_start..]) {
            results.push(comp);
        }
    }

    results
}

/// Parse the argument list that follows the opening `(` of a `component!` call.
fn parse_component_call(s: &str) -> Option<ScannedComponent> {
    let s = s.trim_start();

    let role_end = s.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    let role = s[..role_end].trim().to_string();
    if role.is_empty() {
        return None;
    }

    let s = s[role_end..].trim_start();
    let s = s.strip_prefix(',')?.trim_start();

    let (type_expr, rest) = parse_type_expr(s)?;
    let type_expr = type_expr.trim().to_string();
    if type_expr.is_empty() {
        return None;
    }

    let rest = rest.trim_start();
    let rest = rest.strip_prefix(',')?.trim_start();

    let rest = rest.strip_prefix('"')?;
    let label_end = rest.find('"')?;
    let label = rest[..label_end].to_string();

    Some(ScannedComponent { role, type_expr, label })
}

/// Read characters from `s`, tracking bracket depth, until a top-level `,`.
fn parse_type_expr(s: &str) -> Option<(String, &str)> {
    let mut angle: i32 = 0;
    let mut round: i32 = 0;
    let mut square: i32 = 0;

    for (i, c) in s.char_indices() {
        match c {
            '<' => angle += 1,
            '>' if angle > 0 => angle -= 1,
            '(' => round += 1,
            ')' if round > 0 => round -= 1,
            '[' => square += 1,
            ']' if square > 0 => square -= 1,
            ',' if angle == 0 && round == 0 && square == 0 => {
                return Some((s[..i].to_string(), &s[i..]));
            }
            ')' if round == 0 => return None,
            _ => {}
        }
    }

    None
}

// ── family! / sort_family! scanner ───────────────────────────────────────────

/// Find all `<marker>!(...)` calls in `content` and parse them. `marker` is the
/// full `"<name>!("` literal (precomputed once per scan).
fn scan_families(content: &str, path: &Path, marker: &str) -> Vec<FamilyDef> {
    let source_module = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(rel) = content[search_from..].find(marker) {
        let body_start = search_from + rel + marker.len();
        search_from = body_start;

        if let Some(end) = find_closing(content[body_start..].as_bytes(), b'(', b')') {
            let body = content[body_start..body_start + end].trim();
            if let Some(def) = parse_family_body(body, source_module.clone()) {
                results.push(def);
            }
        }
    }

    results
}

// ── family body parser ───────────────────────────────────────────────────────

fn parse_family_body(body: &str, source_module: String) -> Option<FamilyDef> {
    let entries = split_top_level_commas(body);

    let mut type_template: Option<String> = None;
    let mut uses: Vec<String> = Vec::new();
    let mut axes: Vec<(String, AxisSpec)> = Vec::new();
    let mut fields: Vec<(String, FieldValue)> = Vec::new();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let (key, sep, value) = split_key_value(entry)?;
        let key = key.trim();
        let value = value.trim();

        if sep == ':' {
            let spec = parse_axis_spec(value)?;
            axes.push((key.to_string(), spec));
        } else {
            // sep == '='
            match key {
                "type" => type_template = Some(value.to_string()),
                "uses" => uses = parse_string_array(value)?,
                _ => {
                    let v = parse_field_value(value)?;
                    fields.push((key.to_string(), v));
                }
            }
        }
    }

    Some(FamilyDef {
        type_template: type_template?,
        axes,
        uses,
        fields,
        source_module,
    })
}

/// Classify an axis/field value into a [`FieldValue`].
///
/// - Leading `"` → `String`
/// - Leading `[` → `StringArray`
/// - `true` / `false` → `Bool`
/// - Otherwise parsed as `Int` (returns `None` on failure)
fn parse_field_value(s: &str) -> Option<FieldValue> {
    let s = s.trim();
    if s.starts_with('"') {
        Some(FieldValue::String(parse_string_literal(s)?))
    } else if s.starts_with('[') {
        Some(FieldValue::StringArray(parse_string_array(s)?))
    } else if s == "true" {
        Some(FieldValue::Bool(true))
    } else if s == "false" {
        Some(FieldValue::Bool(false))
    } else if let Ok(n) = s.parse::<i64>() {
        Some(FieldValue::Int(n))
    } else if is_ident(s) {
        Some(FieldValue::Ident(s.to_string()))
    } else {
        None
    }
}

/// True if `s` looks like a Rust identifier — first char is letter/underscore,
/// rest are letters/digits/underscores. Used to recognise pass-through
/// keywords like `inherited`.
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return false; };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ── Parsing helpers ──────────────────────────────────────────────────────────

/// Read the leading identifier, skip whitespace, then take the next character
/// as the separator (must be `:` or `=`). Returns `(key, separator, value)`.
///
/// Robust against `::` inside values — only the first character past the
/// identifier-whitespace boundary is consulted.
fn split_key_value(s: &str) -> Option<(&str, char, &str)> {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
        let b = bytes[end];
        if b.is_ascii_alphanumeric() || b == b'_' {
            end += 1;
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let key = &s[..end];
    let rest = s[end..].trim_start();
    let mut chars = rest.char_indices();
    let (i, c) = chars.next()?;
    if c != ':' && c != '=' {
        return None;
    }
    let after = &rest[i + c.len_utf8()..];
    Some((key, c, after))
}

/// Split `s` by top-level commas (outside `<>`, `()`, `[]`, `{}`, strings).
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut angle: i32 = 0;
    let mut round: i32 = 0;
    let mut square: i32 = 0;
    let mut curly: i32 = 0;
    let mut in_string = false;
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == b'"' {
                let back = bytes[..i].iter().rev().take_while(|&&x| x == b'\\').count();
                if back % 2 == 0 {
                    in_string = false;
                }
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'<' => angle += 1,
                b'>' if angle > 0 => angle -= 1,
                b'(' => round += 1,
                b')' if round > 0 => round -= 1,
                b'[' => square += 1,
                b']' if square > 0 => square -= 1,
                b'{' => curly += 1,
                b'}' if curly > 0 => curly -= 1,
                b',' if angle == 0 && round == 0 && square == 0 && curly == 0 => {
                    parts.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

/// Parse a double-quoted string literal at the start of `s`.
fn parse_string_literal(s: &str) -> Option<String> {
    let s = s.trim();
    let s = s.strip_prefix('"')?;
    let mut result = String::new();
    let mut chars = s.chars();
    loop {
        match chars.next()? {
            '"' => return Some(result),
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                't' => result.push('\t'),
                c => {
                    result.push('\\');
                    result.push(c);
                }
            },
            c => result.push(c),
        }
    }
}

/// Parse `["a", "b", …]` into a `Vec<String>`.
fn parse_string_array(s: &str) -> Option<Vec<String>> {
    let s = s.trim();
    if !s.starts_with('[') {
        return None;
    }
    let inner_end = find_closing(s[1..].as_bytes(), b'[', b']')?;
    let inner = &s[1..1 + inner_end];

    let mut result = Vec::new();
    for part in split_top_level_commas(inner) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        result.push(parse_string_literal(part)?);
    }
    Some(result)
}

/// Parse `[("ty", "lbl"), …]` into a `Vec<ComponentDef>`.
fn parse_pair_list(s: &str) -> Option<Vec<ComponentDef>> {
    let s = s.trim();
    if !s.starts_with('[') {
        return None;
    }
    let inner_end = find_closing(s[1..].as_bytes(), b'[', b']')?;
    let inner = &s[1..1 + inner_end];

    let mut result = Vec::new();
    for part in split_top_level_commas(inner) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if !part.starts_with('(') {
            return None;
        }
        let pair_end = find_closing(part[1..].as_bytes(), b'(', b')')?;
        let pair_inner = &part[1..1 + pair_end];
        let sub = split_top_level_commas(pair_inner);
        if sub.len() != 2 {
            return None;
        }
        let ty = parse_string_literal(sub[0].trim())?;
        let lbl = parse_string_literal(sub[1].trim())?;
        result.push(ComponentDef::new(ty, lbl));
    }
    Some(result)
}

/// Parse an axis spec value (the part after `:`):
/// - `RoleName` — `AxisSpec::Role`
/// - `cross(Left, Right, "ty_tmpl", "lbl_tmpl") [+ [...]]` — `AxisSpec::Cross`
/// - `inline [...]` — `AxisSpec::Inline`
fn parse_axis_spec(s: &str) -> Option<AxisSpec> {
    let s = s.trim();
    if s.starts_with("cross") {
        let s = s["cross".len()..].trim_start();
        if !s.starts_with('(') {
            return None;
        }
        let arg_end = find_closing(s[1..].as_bytes(), b'(', b')')?;
        let args_str = &s[1..1 + arg_end];
        let after = s[1 + arg_end + 1..].trim();

        let args = split_top_level_commas(args_str);
        if args.len() != 4 {
            return None;
        }
        let left = args[0].trim().to_string();
        let right = args[1].trim().to_string();
        let type_tmpl = parse_string_literal(args[2].trim())?;
        let label_tmpl = parse_string_literal(args[3].trim())?;

        let extras = if let Some(rest) = after.strip_prefix('+') {
            parse_pair_list(rest.trim())?
        } else {
            Vec::new()
        };

        Some(AxisSpec::Cross { left, right, type_tmpl, label_tmpl, extras })
    } else if s.starts_with("inline") {
        let s = s["inline".len()..].trim_start();
        Some(AxisSpec::Inline(parse_pair_list(s)?))
    } else {
        Some(AxisSpec::Role(s.to_string()))
    }
}

/// Find the matching `close` byte in `bytes`, assuming we start just after the
/// opening `open` byte (depth starts at 1). Returns the index of `close`.
/// Correctly skips over string literals delimited by `"…"`.
fn find_closing(bytes: &[u8], open: u8, close: u8) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut in_string = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == b'"' {
                let back = bytes[..i].iter().rev().take_while(|&&x| x == b'\\').count();
                if back % 2 == 0 {
                    in_string = false;
                }
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn marker() -> String {
        "family!(".to_string()
    }

    #[test]
    fn simple_type() {
        let result = scan_components(r#"combo_codegen::component!(Partition, Lomuto, "lomuto");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "Partition");
        assert_eq!(result[0].type_expr, "Lomuto");
        assert_eq!(result[0].label, "lomuto");
    }

    #[test]
    fn const_generic_type() {
        let result =
            scan_components(r#"component!(SmallSort, InsertionSmallSort<16>, "insertion: 16");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_expr, "InsertionSmallSort<16>");
    }

    #[test]
    fn nested_generics() {
        let result = scan_components(r#"component!(MyRole, Outer<Inner<u8>>, "nested");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_expr, "Outer<Inner<u8>>");
    }

    #[test]
    fn multiple_in_file() {
        let src = r#"
            component!(Partition, Lomuto, "lomuto");
            component!(Partition, Hoare, "hoare");
            component!(PivotSelector, FirstElement, "first");
        "#;
        let result = scan_components(src);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn trailing_comma() {
        let result = scan_components(r#"component!(R, SomeType, "label",);"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "label");
    }

    #[test]
    fn qualified_path() {
        let result = scan_components(
            r#"combo_codegen::component!(Rotation, ReversalRotation, "reversal");"#,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "Rotation");
    }

    #[test]
    fn boolean_const_generic() {
        let result = scan_components(r#"component!(PingPong, true, "ping-pong");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_expr, "true");
    }

    #[test]
    fn parse_simple_family() {
        let src = r#"combo_codegen::family!(
            type = MySort<{A}, {B}>,
            uses = ["crate::foo::Bar", "crate::baz::Qux"],
            A: RoleA,
            B: RoleB,
            name = "my sort",
            big_o = "O(N log N)",
            stable = true,
            direct_sort = false,
            path = ["root", "{A}", "{B}"],
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/sorts/my_sorts/foo.rs"), &marker());
        assert_eq!(defs.len(), 1);
        let d = &defs[0];
        assert_eq!(d.type_template, "MySort<{A}, {B}>");
        assert_eq!(d.uses, vec!["crate::foo::Bar", "crate::baz::Qux"]);
        assert_eq!(d.axes.len(), 2);
        assert!(matches!(&d.axes[0].1, AxisSpec::Role(r) if r == "RoleA"));
        // Five trailing fields: name, big_o, stable, direct_sort, path.
        assert_eq!(d.fields.len(), 5);
        assert_eq!(d.fields[0].0, "name");
        assert!(matches!(&d.fields[0].1, FieldValue::String(s) if s == "my sort"));
        assert!(matches!(&d.fields[2].1, FieldValue::Bool(true)));
        assert!(matches!(&d.fields[3].1, FieldValue::Bool(false)));
        assert!(matches!(&d.fields[4].1, FieldValue::StringArray(a) if a.len() == 3));
        assert_eq!(d.source_module, "my_sorts");
    }

    #[test]
    fn parse_inline_axis() {
        let src = r#"family!(
            type = S<{PP}>,
            uses = [],
            PP: inline [("false", ""), ("true", "ping-pong")],
            name = "s",
            big_o = "O(1)",
            stable = true,
            direct_sort = true,
            path = ["s"],
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/sorts/foo/bar.rs"), &marker());
        assert_eq!(defs.len(), 1);
        let ax = &defs[0].axes[0];
        assert_eq!(ax.0, "PP");
        if let AxisSpec::Inline(items) = &ax.1 {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].type_expr, "false");
            assert_eq!(items[1].label, "ping-pong");
        } else {
            panic!("expected Inline axis");
        }
    }

    #[test]
    fn parse_cross_axis_with_extras() {
        let src = r#"family!(
            type = DS<{DPS}>,
            uses = [],
            DPS: cross(PivotSelector, PivotSelector, "CombinedSelector<{0}, {1}>", "{0} / {1}")
               + [("NintherDualPivot", "ninther")],
            name = "dual",
            big_o = "O(N log N)",
            stable = false,
            direct_sort = true,
            path = ["dual"],
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/sorts/qs/dp.rs"), &marker());
        assert_eq!(defs.len(), 1);
        let ax = &defs[0].axes[0];
        if let AxisSpec::Cross { left, right, type_tmpl, label_tmpl, extras } = &ax.1 {
            assert_eq!(left, "PivotSelector");
            assert_eq!(right, "PivotSelector");
            assert_eq!(type_tmpl, "CombinedSelector<{0}, {1}>");
            assert_eq!(label_tmpl, "{0} / {1}");
            assert_eq!(extras.len(), 1);
            assert_eq!(extras[0].type_expr, "NintherDualPivot");
        } else {
            panic!("expected Cross axis");
        }
    }

    #[test]
    fn parse_int_field() {
        let src = r#"family!(
            type = X<{A}>,
            uses = [],
            A: RoleA,
            name = "x",
            big_o = "O(1)",
            stable = true,
            direct_sort = true,
            max_n_for_tests = 200,
            path = ["x"],
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/foo/bar.rs"), &marker());
        let d = &defs[0];
        let m = d.fields.iter().find(|(k, _)| k == "max_n_for_tests").unwrap();
        assert!(matches!(&m.1, FieldValue::Int(200)));
    }

    #[test]
    fn separator_robust_to_double_colon() {
        // Ensure split_key_value uses the FIRST char past whitespace, not the
        // first ':' or '=' encountered at top level.
        let (k, sep, v) = split_key_value("name = \"hello :: world\"").unwrap();
        assert_eq!(k, "name");
        assert_eq!(sep, '=');
        assert_eq!(v.trim(), "\"hello :: world\"");
    }

    #[test]
    fn alternative_marker() {
        let src = r#"family!(
            type = M<{A}>,
            uses = [],
            A: RoleA,
            name = "m",
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/x/y.rs"), "family!(");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].type_template, "M<{A}>");
    }
}
