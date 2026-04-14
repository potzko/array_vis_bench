use std::path::{Path, PathBuf};

use crate::family::ComponentRegistry;

/// A single `component!(Role, TypeExpr, "label")` call found in a source file.
#[derive(Debug, Clone)]
struct ScannedComponent {
    role: String,
    type_expr: String,
    label: String,
}

/// Returned by [`scan`]. Holds the discovered registry and the list of files
/// that were read so the caller can emit `cargo:rerun-if-changed` lines.
pub struct ScanResult {
    pub registry: ComponentRegistry,
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
}

/// Recursively walk `dir`, parse every `.rs` file for `component!(...)` calls,
/// and return the aggregated [`ScanResult`].
pub fn scan(dir: impl AsRef<Path>) -> Result<ScanResult, std::io::Error> {
    let mut registry = ComponentRegistry::default();
    let mut scanned_files = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path())?;
        for comp in scan_file(&content) {
            registry.add(comp.role, comp.type_expr, comp.label);
        }
        scanned_files.push(entry.path().to_path_buf());
    }

    Ok(ScanResult { registry, scanned_files })
}

// ── File-level parser ────────────────────────────────────────────────────────

/// Find all `component!(...)` calls in `content` and return parsed components.
/// Calls that cannot be parsed (wrong syntax, inside strings, etc.) are
/// silently skipped — false positives are effectively impossible given how
/// specific the three-argument signature is.
fn scan_file(content: &str) -> Vec<ScannedComponent> {
    const MARKER: &str = "component!(";
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(rel) = content[search_from..].find(MARKER) {
        let args_start = search_from + rel + MARKER.len();
        search_from = args_start; // advance so the next iteration starts here
        if let Some(comp) = parse_component_call(&content[args_start..]) {
            results.push(comp);
        }
    }

    results
}

// ── Argument parser ──────────────────────────────────────────────────────────

/// Parse the argument list that follows the opening `(` of a `component!` call:
///
/// ```text
/// Role, TypeExpr<with<const, generics>>, "label"[,])
/// ```
///
/// Returns `None` if the input doesn't match the expected shape.
fn parse_component_call(s: &str) -> Option<ScannedComponent> {
    let s = s.trim_start();

    // ── arg 1: role identifier ───────────────────────────────────────────────
    let role_end = s.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    let role = s[..role_end].trim().to_string();
    if role.is_empty() {
        return None;
    }

    let s = s[role_end..].trim_start();
    let s = s.strip_prefix(',')?.trim_start();

    // ── arg 2: type expression (may contain balanced <>, (), []) ────────────
    let (type_expr, rest) = parse_type_expr(s)?;
    let type_expr = type_expr.trim().to_string();
    if type_expr.is_empty() {
        return None;
    }

    let rest = rest.trim_start();
    let rest = rest.strip_prefix(',')?.trim_start();

    // ── arg 3: string literal ────────────────────────────────────────────────
    let rest = rest.strip_prefix('"')?;
    let label_end = rest.find('"')?;
    let label = rest[..label_end].to_string();

    Some(ScannedComponent { role, type_expr, label })
}

/// Read characters from `s`, tracking bracket depth, until a top-level `,` is
/// found. Returns `(type_expr_text, rest_from_comma)`.
///
/// Brackets tracked: `<>` (generics / const generics), `()`, `[]`.
/// A `>` that would underflow the angle-bracket depth is treated as a
/// comparison operator and does not close a bracket — this is deliberately
/// lenient so that expressions like `N > 0` in const-generic positions don't
/// confuse the parser.
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
            // Top-level comma — this is the separator between arg 2 and arg 3.
            ',' if angle == 0 && round == 0 && square == 0 => {
                return Some((s[..i].to_string(), &s[i..]));
            }
            // Top-level closing paren — end of macro call with no separator found.
            ')' if round == 0 => return None,
            _ => {}
        }
    }

    None // ran off the end without finding a top-level comma
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_type() {
        let result = scan_file(r#"combo_codegen::component!(Partition, Lomuto, "lomuto");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "Partition");
        assert_eq!(result[0].type_expr, "Lomuto");
        assert_eq!(result[0].label, "lomuto");
    }

    #[test]
    fn const_generic_type() {
        let result = scan_file(r#"component!(SmallSort, InsertionSmallSort<16>, "insertion: 16");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_expr, "InsertionSmallSort<16>");
    }

    #[test]
    fn nested_generics() {
        let result = scan_file(r#"component!(MyRole, Outer<Inner<u8>>, "nested");"#);
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
        let result = scan_file(src);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn trailing_comma() {
        // Rust allows trailing commas in macro calls.
        // Our parser stops at the label-closing quote and doesn't need to
        // consume the optional trailing comma, so this should still parse.
        let result = scan_file(r#"component!(R, SomeType, "label",);"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "label");
    }

    #[test]
    fn qualified_path() {
        let result =
            scan_file(r#"combo_codegen::component!(Rotation, ReversalRotation, "reversal");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "Rotation");
    }

    #[test]
    fn boolean_const_generic() {
        // Inline axes (e.g. true/false for ping-pong) use plain identifiers.
        let result = scan_file(r#"component!(PingPong, true, "ping-pong");"#);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].type_expr, "true");
    }
}
