//! Stage 4 — emit Rust source from resolved specs.

use std::collections::BTreeSet;

use crate::registry::Registry;
use crate::resolve::{resolve, Resolved};
use crate::spec::SpecNode;

fn use_lines(uses: &[String], indent: &str) -> String {
    uses.iter().map(|u| format!("{indent}use {u};\n")).collect()
}

/// One spec → a named type alias + derived label + dispatch fn. Each sort's
/// imports are scoped inside its own private module, so two invocations whose
/// `uses` overlap don't collide at crate scope (a real hazard — duplicate
/// `use` at module scope is a compile error). The public names are re-exported.
pub fn emit_one(alias: &str, r: &Resolved) -> String {
    format!(
        "#[allow(non_snake_case)]\n\
         mod __{alias}_impl {{\n\
         \x20   #[allow(unused_imports)] use super::*;\n{uses}\
         \x20   pub type Ty = {ty};\n\
         \x20   pub const NAME: &str = {label:?};\n\
         \x20   pub fn run(arr: &mut [usize]) {{ <Ty>::sort(arr); }}\n\
         }}\n\
         pub use __{alias}_impl::Ty as {alias};\n\
         #[allow(non_upper_case_globals)] pub use __{alias}_impl::NAME as {alias}_NAME;\n\
         #[allow(non_snake_case)] pub use __{alias}_impl::run as {alias}_run;\n",
        uses = use_lines(&r.uses, "    "),
        ty = r.type_expr,
        label = r.label,
    )
}

/// Many specs → one module with a `SORTS` dispatch table. The union of every
/// variant's imports is emitted once at the top (deduped), so the table needs
/// no per-row scoping.
pub fn generate_table(reg: &Registry, specs: &[SpecNode], module: &str) -> Result<String, String> {
    let mut all_uses: BTreeSet<String> = BTreeSet::new();
    let mut aliases = String::new();
    let mut rows = String::new();
    for (i, spec) in specs.iter().enumerate() {
        let r = resolve(spec, reg)?;
        all_uses.extend(r.uses.iter().cloned());
        aliases.push_str(&format!(
            "    pub type S{i} = {ty};\n    pub fn s{i}(arr: &mut [usize]) {{ <S{i}>::sort(arr); }}\n",
            ty = r.type_expr,
        ));
        rows.push_str(&format!("        ({label:?}, s{i}),\n", label = r.label));
    }
    let uses: Vec<String> = all_uses.into_iter().collect();
    Ok(format!(
        "pub mod {module} {{\n    #[allow(unused_imports)] use super::*;\n{uses}{aliases}\
         \n    pub const SORTS: &[(&str, fn(&mut [usize]))] = &[\n{rows}    ];\n}}\n",
        uses = use_lines(&uses, "    "),
    ))
}
