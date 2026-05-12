use std::path::{Path, PathBuf};

use crate::family::{AxisSpec, ComponentDef, ComponentRegistry, SortFamilyDef};

/// A single `component!(Role, TypeExpr, "label")` call found in a source file.
#[derive(Debug, Clone)]
struct ScannedComponent {
    role: String,
    type_expr: String,
    label: String,
}

/// Returned by [`scan`]. Holds the discovered registry, scanned sort families,
/// and the list of files that were read so the caller can emit
/// `cargo:rerun-if-changed` lines.
pub struct ScanResult {
    pub registry: ComponentRegistry,
    pub families: Vec<SortFamilyDef>,
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

    /// Generate one `*_combinations.rs` file per source module into `out_dir`.
    ///
    /// Families are grouped by their [`SortFamilyDef::source_module`].  Within
    /// each group, `use` declarations are deduplicated (first-occurrence order)
    /// and the resolved `sort_registry_macro::sort_family! { … }` blocks are
    /// appended. Each block carries its `group_size` (leaf count) so the
    /// runtime can surface specialised (small-group) sorts first.
    pub fn emit_sort_families(&self, out_dir: &Path) -> Result<(), std::io::Error> {
        // Collect unique module names in first-occurrence order.
        let mut modules: Vec<&str> = Vec::new();
        for fam in &self.families {
            let m = fam.source_module.as_str();
            if !modules.contains(&m) {
                modules.push(m);
            }
        }

        for module in modules {
            let fams: Vec<&SortFamilyDef> = self
                .families
                .iter()
                .filter(|f| f.source_module == module)
                .collect();

            let mut out = String::new();

            // Emit deduplicated `use` statements (first-occurrence order).
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

            // Emit each family block.
            for fam in &fams {
                fam.render(&mut out, &self.registry);
            }

            let filename = format!("{}_combinations.rs", module);
            std::fs::write(out_dir.join(&filename), &out)?;
        }

        Ok(())
    }
}

/// Recursively walk `dir`, parse every `.rs` file for `component!(...)` and
/// `sort_family!(...)` calls, and return the aggregated [`ScanResult`].
///
/// Files are visited in lexicographic path order so the scan — and every
/// downstream piece of generated code — is reproducible across machines and
/// filesystems.
pub fn scan(dir: impl AsRef<Path>) -> Result<ScanResult, std::io::Error> {
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

    for path in paths {
        let content = std::fs::read_to_string(&path)?;
        for comp in scan_components(&content) {
            registry.add(comp.role, comp.type_expr, comp.label);
        }
        families.extend(scan_families(&content, &path));
        scanned_files.push(path);
    }

    Ok(ScanResult { registry, families, scanned_files })
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

// ── sort_family! scanner ─────────────────────────────────────────────────────

/// Find all `sort_family!(...)` calls in `content` and parse them.
fn scan_families(content: &str, path: &Path) -> Vec<SortFamilyDef> {
    let source_module = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    const MARKER: &str = "sort_family!(";
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(rel) = content[search_from..].find(MARKER) {
        let body_start = search_from + rel + MARKER.len();
        search_from = body_start;

        if let Some(end) = find_closing(content[body_start..].as_bytes(), b'(', b')') {
            let body = content[body_start..body_start + end].trim();
            if let Some(def) = parse_sort_family_body(body, source_module.clone()) {
                results.push(def);
            }
        }
    }

    results
}

// ── sort_family! body parser ─────────────────────────────────────────────────

fn parse_sort_family_body(body: &str, source_module: String) -> Option<SortFamilyDef> {
    let fields = split_top_level_commas(body);

    let mut type_template: Option<String> = None;
    let mut uses: Vec<String> = Vec::new();
    let mut axes: Vec<(String, AxisSpec)> = Vec::new();
    let mut name: Option<String> = None;
    let mut big_o: Option<String> = None;
    let mut stable: Option<bool> = None;
    let mut direct_sort: Option<bool> = None;
    let mut path: Option<Vec<String>> = None;
    let mut max_n_for_tests: Option<u64> = None;

    for field in fields {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }

        let (raw_key, raw_value) = split_key_value(field)?;
        let key = raw_key.trim();
        let value = raw_value.trim();

        match key {
            "type" => type_template = Some(value.to_string()),
            "uses" => uses = parse_string_array(value)?,
            "name" => name = Some(parse_string_literal(value)?),
            "big_o" => big_o = Some(parse_string_literal(value)?),
            "stable" => stable = Some(value == "true"),
            "direct_sort" => direct_sort = Some(value == "true"),
            "path" => path = Some(parse_string_array(value)?),
            "max_n_for_tests" => max_n_for_tests = Some(value.trim().parse().ok()?),
            var => {
                let spec = parse_axis_spec(value)?;
                axes.push((var.to_string(), spec));
            }
        }
    }

    Some(SortFamilyDef {
        type_template: type_template?,
        axes,
        uses,
        name: name?,
        big_o: big_o?,
        stable: stable?,
        direct_sort: direct_sort?,
        path: path?,
        max_n_for_tests,
        source_module,
    })
}

// ── Parsing helpers ──────────────────────────────────────────────────────────

/// Split `s` on the first `=` or `:` that sits at top-level depth
/// (outside `<>`, `()`, `[]`, strings).
fn split_key_value(s: &str) -> Option<(&str, &str)> {
    let mut angle: i32 = 0;
    let mut round: i32 = 0;
    let mut square: i32 = 0;
    let mut in_string = false;

    for (i, c) in s.char_indices() {
        match c {
            '"' if !in_string => in_string = true,
            '"' if in_string => in_string = false,
            _ if in_string => {}
            '<' => angle += 1,
            '>' if angle > 0 => angle -= 1,
            '(' => round += 1,
            ')' if round > 0 => round -= 1,
            '[' => square += 1,
            ']' if square > 0 => square -= 1,
            '=' | ':' if angle == 0 && round == 0 && square == 0 => {
                return Some((&s[..i], &s[i + c.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}

/// Split `s` by top-level commas (outside `<>`, `()`, `[]`, `{}`, strings).
/// Returns a `Vec` of `&str` slices (not trimmed).
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
                // check for escaped quote
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
/// Returns the unescaped content.
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
    let inner_end = find_closing(&s[1..].as_bytes(), b'[', b']')?;
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
        // Plain role name.
        Some(AxisSpec::Role(s.to_string()))
    }
}

/// Find the matching `close` byte in `bytes`, assuming we start just after the
/// opening `open` byte (depth starts at 1).  Returns the index of `close`.
///
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

    // ── sort_family! tests ───────────────────────────────────────────────────

    #[test]
    fn parse_simple_family() {
        let src = r#"combo_codegen::sort_family!(
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
        let defs = scan_families(src, std::path::Path::new("src/sorts/my_sorts/foo.rs"));
        assert_eq!(defs.len(), 1);
        let d = &defs[0];
        assert_eq!(d.type_template, "MySort<{A}, {B}>");
        assert_eq!(d.uses, vec!["crate::foo::Bar", "crate::baz::Qux"]);
        assert_eq!(d.axes.len(), 2);
        assert_eq!(d.axes[0].0, "A");
        assert!(matches!(&d.axes[0].1, AxisSpec::Role(r) if r == "RoleA"));
        assert_eq!(d.name, "my sort");
        assert_eq!(d.big_o, "O(N log N)");
        assert!(d.stable);
        assert!(!d.direct_sort);
        assert_eq!(d.path, vec!["root", "{A}", "{B}"]);
        assert_eq!(d.source_module, "my_sorts");
    }

    #[test]
    fn parse_inline_axis() {
        let src = r#"sort_family!(
            type = S<{PP}>,
            uses = [],
            PP: inline [("false", ""), ("true", "ping-pong")],
            name = "s",
            big_o = "O(1)",
            stable = true,
            direct_sort = true,
            path = ["s"],
        );"#;
        let defs = scan_families(src, std::path::Path::new("src/sorts/foo/bar.rs"));
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
        let src = r#"sort_family!(
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
        let defs = scan_families(src, std::path::Path::new("src/sorts/qs/dp.rs"));
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
}
