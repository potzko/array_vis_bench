//! Stage 4 — emit Rust source from resolved specs.

use crate::registry::Registry;
use crate::resolve::{resolve, Resolved};
use crate::spec::SpecNode;

/// One spec → a named type alias + derived label + dispatch fn. Used by the
/// inline macro front-end. In the real crate this is where the `AlgorithmEntry`
/// struct literal + `#[linkme::distributed_slice(ALGORITHMS)]` static go.
pub fn emit_one(alias: &str, r: &Resolved) -> String {
    format!(
        "pub type {alias} = {ty};\n\
         #[allow(non_upper_case_globals)] pub const {alias}_NAME: &str = {label:?};\n\
         #[allow(non_snake_case)] pub fn {alias}_run(arr: &mut [usize]) {{ <{alias}>::sort(arr); }}\n",
        ty = r.type_expr,
        label = r.label,
    )
}

/// Many specs → one module with a `SORTS` dispatch table. Used by the generator
/// (build-time or standalone). Each spec is resolved independently; an
/// unresolvable spec is an error (the caller decides whether to skip).
pub fn generate_table(reg: &Registry, specs: &[SpecNode], module: &str) -> Result<String, String> {
    let mut aliases = String::new();
    let mut rows = String::new();
    for (i, spec) in specs.iter().enumerate() {
        let r = resolve(spec, reg)?;
        aliases.push_str(&format!(
            "    pub type S{i} = {ty};\n    pub fn s{i}(arr: &mut [usize]) {{ <S{i}>::sort(arr); }}\n",
            ty = r.type_expr,
        ));
        rows.push_str(&format!("        ({label:?}, s{i}),\n", label = r.label));
    }
    Ok(format!(
        "pub mod {module} {{\n    use super::*;\n{aliases}\
         \n    pub const SORTS: &[(&str, fn(&mut [usize]))] = &[\n{rows}    ];\n}}\n"
    ))
}
