//! Inline front-end (mode 1: "generate a sort"). A thin shell over `spec_core`:
//! it turns the macro input into text, runs the engine, and parses the emitted
//! Rust back into tokens. All real work — parse, resolve, role-check, emit —
//! lives in `spec_core` and is shared with the generator.
//!
//! ```ignore
//! sort_spec!(QuickLLMidIns32 = quick_sort<
//!     small_sort = insertion_sort<32>
//!     partition  = LL_partition< pivot = middle_element >
//! >);
//! ```

use proc_macro::TokenStream;

/// The registry is compiled in for the prototype. In production this is the
/// `Cargo.toml` metadata, read once in build.rs.
const REGISTRY: &str = include_str!("../../registry.spec");

#[proc_macro]
pub fn sort_spec(input: TokenStream) -> TokenStream {
    match compile(&input.to_string()) {
        Ok(src) => src.parse().expect("emitted Rust should parse"),
        Err(msg) => format!("compile_error!({:?});", msg)
            .parse()
            .expect("compile_error! should parse"),
    }
}

fn compile(input: &str) -> Result<String, String> {
    let reg = spec_core::Registry::parse(REGISTRY)?;
    let (alias, node) = spec_core::parse_spec(input)?;
    let alias = alias.ok_or("expected `Alias = <spec tree>`")?;
    let resolved = spec_core::resolve(&node, &reg)?;
    Ok(spec_core::emit_one(&alias, &resolved))
}
