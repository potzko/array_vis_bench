use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitBool, LitInt, LitStr, Result, Token,
};

// ============================================================
// AST types
// ============================================================

pub(crate) struct SortFamilyInput {
    sort_template: TypeExpr,
    axes: Vec<AxisDef>,
    name: String,
    big_o: String,
    stable: bool,
    path_template: Vec<String>,
    /// When true: call `ConcreteType::sort(arr, logger)` directly (inherent method).
    /// Also generates a `sort_vis` fn and registers in `SORT_VIS_REGISTRY`.
    /// Use for sorts whose inherent sort method accepts `U: ?Sized + SortLogger<T>`.
    ///
    /// When false (default): call via `<ConcreteType as SortAlgo<usize, NoOpLogger>>::sort`.
    /// Use for sorts that implement `SortAlgo`.
    direct_sort: bool,
    /// Optional upper bound on random-input array size in correctness tests.
    /// Slow sorts set this to skip pathological cases.
    max_n_for_tests: Option<u64>,
}

struct AxisDef {
    slot: String,
    variants: Vec<VariantNode>,
}

struct VariantNode {
    ty: TypeExpr,
    label: String,
    sub_axes: Vec<AxisDef>,
}

/// A type expression:
/// - `Name`                    — a plain ident (possibly zero generic args)
/// - `Name<arg, arg, ...>`     — a generic type with arguments
/// - `{SlotName}`              — a bare slot that stands for the whole type
/// - `true` / `false`          — bool const generic
/// - integer literal            — integer const generic
///
/// Arguments can be `{Slot}` placeholders, nested TypeExprs, bool literals,
/// or integer literals (const generic parameters).
enum TypeExpr {
    Named(Ident, Vec<TypeExprArg>),
    Slot(String),
    Bool(bool),
    Int(LitInt),
}

enum TypeExprArg {
    Slot(String),
    Type(TypeExpr),
    Bool(bool),
    Int(LitInt),
}

// ============================================================
// Parsing
// ============================================================

fn parse_type_expr(input: ParseStream) -> Result<TypeExpr> {
    // A type expression either starts with `{` (bare slot), a bool/int literal, or an Ident
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let slot: Ident = content.parse()?;
        return Ok(TypeExpr::Slot(slot.to_string()));
    }

    if input.peek(LitBool) {
        let b: LitBool = input.parse()?;
        return Ok(TypeExpr::Bool(b.value()));
    }

    if input.peek(LitInt) {
        let n: LitInt = input.parse()?;
        return Ok(TypeExpr::Int(n));
    }

    let name: Ident = input.parse()?;
    let args = if input.peek(Token![<]) {
        let _: Token![<] = input.parse()?;
        let mut args = vec![];
        while !input.peek(Token![>]) {
            args.push(parse_type_expr_arg(input)?);
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }
        let _: Token![>] = input.parse()?;
        args
    } else {
        vec![]
    };

    Ok(TypeExpr::Named(name, args))
}

fn parse_type_expr_arg(input: ParseStream) -> Result<TypeExprArg> {
    if input.peek(syn::token::Brace) {
        // {SlotName}
        let content;
        braced!(content in input);
        let slot: Ident = content.parse()?;
        Ok(TypeExprArg::Slot(slot.to_string()))
    } else if input.peek(LitBool) {
        let b: LitBool = input.parse()?;
        Ok(TypeExprArg::Bool(b.value()))
    } else if input.peek(LitInt) {
        let n: LitInt = input.parse()?;
        Ok(TypeExprArg::Int(n))
    } else {
        Ok(TypeExprArg::Type(parse_type_expr(input)?))
    }
}

fn parse_axis_def(input: ParseStream) -> Result<AxisDef> {
    let slot: Ident = input.parse()?;
    let content;
    braced!(content in input);
    let mut variants = vec![];
    while !content.is_empty() {
        variants.push(parse_variant_node(&content)?);
    }
    Ok(AxisDef { slot: slot.to_string(), variants })
}

fn parse_variant_node(input: ParseStream) -> Result<VariantNode> {
    let ty = parse_type_expr(input)?;
    let _: Token![=>] = input.parse()?;
    let label: LitStr = input.parse()?;
    let sub_axes = if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let mut axes = vec![];
        while !content.is_empty() {
            axes.push(parse_axis_def(&content)?);
        }
        axes
    } else {
        vec![]
    };
    Ok(VariantNode { ty, label: label.value(), sub_axes })
}

impl Parse for SortFamilyInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // type Sort = TypeExpr;
        let _: Token![type] = input.parse()?;
        let _sort: Ident = input.parse()?; // "Sort"
        let _: Token![=] = input.parse()?;
        let sort_template = parse_type_expr(input)?;
        let _: Token![;] = input.parse()?;

        let mut axes: Vec<AxisDef> = vec![];
        let mut name: Option<String> = None;
        let mut big_o: Option<String> = None;
        let mut stable: Option<bool> = None;
        let mut path_template: Vec<String> = vec![];
        let mut direct_sort: bool = false;
        let mut max_n_for_tests: Option<u64> = None;

        while !input.is_empty() {
            if input.peek(Ident) && input.peek2(syn::token::Brace) {
                axes.push(parse_axis_def(input)?);
            } else if input.peek(Ident) && input.peek2(Token![=]) {
                let field: Ident = input.parse()?;
                let _: Token![=] = input.parse()?;
                match field.to_string().as_str() {
                    "name" => {
                        name = Some(input.parse::<LitStr>()?.value());
                        let _: Token![;] = input.parse()?;
                    }
                    "big_o" => {
                        big_o = Some(input.parse::<LitStr>()?.value());
                        let _: Token![;] = input.parse()?;
                    }
                    "stable" => {
                        stable = Some(input.parse::<LitBool>()?.value());
                        let _: Token![;] = input.parse()?;
                    }
                    "path" => {
                        let content;
                        bracketed!(content in input);
                        let parts: Punctuated<LitStr, Token![,]> =
                            Punctuated::parse_terminated(&content)?;
                        path_template = parts.iter().map(|s| s.value()).collect();
                        let _: Token![;] = input.parse()?;
                    }
                    "direct_sort" => {
                        direct_sort = input.parse::<LitBool>()?.value();
                        let _: Token![;] = input.parse()?;
                    }
                    "max_n_for_tests" => {
                        max_n_for_tests = Some(input.parse::<LitInt>()?.base10_parse::<u64>()?);
                        let _: Token![;] = input.parse()?;
                    }
                    other => {
                        return Err(syn::Error::new(
                            field.span(),
                            format!(
                                "Unknown field `{other}`. Expected: name, big_o, stable, path, direct_sort, max_n_for_tests"
                            ),
                        ));
                    }
                }
            } else {
                return Err(input.error(
                    "Expected an axis definition `SlotName { ... }` or a meta field `name = ...;`",
                ));
            }
        }

        Ok(SortFamilyInput {
            sort_template,
            axes,
            name: name.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "sort_family!: missing `name = ...;`")
            })?,
            big_o: big_o.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "sort_family!: missing `big_o = ...;`")
            })?,
            stable: stable.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "sort_family!: missing `stable = ...;`")
            })?,
            path_template,
            direct_sort,
            max_n_for_tests,
        })
    }
}

// ============================================================
// Enumeration
// ============================================================

struct Leaf {
    /// Fully substituted concrete type tokens
    concrete_ty: TokenStream2,
    /// slot_name → label (for path template substitution)
    slot_labels: HashMap<String, String>,
    /// All non-empty labels in traversal order (for sort name suffix)
    labels: Vec<String>,
}

fn render_type(ty: &TypeExpr, bindings: &HashMap<String, TokenStream2>) -> TokenStream2 {
    match ty {
        TypeExpr::Slot(s) => bindings
            .get(s)
            .cloned()
            .unwrap_or_else(|| quote! { compile_error!("sort_family!: unresolved slot") }),
        TypeExpr::Named(name, args) => {
            if args.is_empty() {
                quote! { #name }
            } else {
                let rendered: Vec<_> = args.iter().map(|a| render_arg(a, bindings)).collect();
                quote! { #name < #(#rendered),* > }
            }
        }
        TypeExpr::Bool(b) => quote! { #b },
        TypeExpr::Int(n) => quote! { #n },
    }
}

fn render_arg(arg: &TypeExprArg, bindings: &HashMap<String, TokenStream2>) -> TokenStream2 {
    match arg {
        TypeExprArg::Slot(s) => bindings
            .get(s)
            .cloned()
            .unwrap_or_else(|| quote! { compile_error!("sort_family!: unresolved slot") }),
        TypeExprArg::Type(t) => render_type(t, bindings),
        TypeExprArg::Bool(b) => quote! { #b },
        TypeExprArg::Int(n) => quote! { #n },
    }
}

/// Enumerate all concrete combinations for a single axis.
///
/// Returns `Vec<(slot_bindings, slot_labels, labels)>` where:
/// - `slot_bindings`: this axis's slot → concrete type tokens (ready to substitute)
/// - `slot_labels`: ALL slot names in this subtree → their label strings
/// - `labels`: collected non-empty labels in traversal order
fn enumerate_axis(
    axis: &AxisDef,
) -> Vec<(HashMap<String, TokenStream2>, HashMap<String, String>, Vec<String>)> {
    let mut results = vec![];

    for variant in &axis.variants {
        let lbl = variant.label.clone();

        if variant.sub_axes.is_empty() {
            // Terminal: variant type is fully concrete
            let ty = render_type(&variant.ty, &HashMap::new());
            let mut bindings = HashMap::new();
            bindings.insert(axis.slot.clone(), ty);
            let mut slot_labels = HashMap::new();
            slot_labels.insert(axis.slot.clone(), lbl.clone());
            let labels = if lbl.is_empty() { vec![] } else { vec![lbl] };
            results.push((bindings, slot_labels, labels));
        } else {
            // Sub-axes form a cross-product; substitute into this variant's type
            for (sub_bind, mut sub_sl, sub_labels) in cross_product_axes(&variant.sub_axes) {
                let ty = render_type(&variant.ty, &sub_bind);
                let mut bindings = HashMap::new();
                bindings.insert(axis.slot.clone(), ty);
                sub_sl.insert(axis.slot.clone(), lbl.clone());
                let mut labels = if lbl.is_empty() { vec![] } else { vec![lbl.clone()] };
                labels.extend(sub_labels);
                results.push((bindings, sub_sl, labels));
            }
        }
    }

    results
}

/// Cross-product of a list of axes.
fn cross_product_axes(
    axes: &[AxisDef],
) -> Vec<(HashMap<String, TokenStream2>, HashMap<String, String>, Vec<String>)> {
    let mut acc: Vec<(HashMap<String, TokenStream2>, HashMap<String, String>, Vec<String>)> =
        vec![(HashMap::new(), HashMap::new(), vec![])];

    for axis in axes {
        let leaves = enumerate_axis(axis);
        let mut next = vec![];
        for (pb, psl, pl) in acc {
            for (lb, lsl, ll) in &leaves {
                let mut b = pb.clone();
                b.extend(lb.clone());
                let mut sl = psl.clone();
                sl.extend(lsl.clone());
                let mut l = pl.clone();
                l.extend(ll.clone());
                next.push((b, sl, l));
            }
        }
        acc = next;
    }

    acc
}

fn enumerate(family: &SortFamilyInput) -> Vec<Leaf> {
    if family.axes.is_empty() {
        let ty = render_type(&family.sort_template, &HashMap::new());
        return vec![Leaf { concrete_ty: ty, slot_labels: HashMap::new(), labels: vec![] }];
    }

    cross_product_axes(&family.axes)
        .into_iter()
        .map(|(bindings, slot_labels, labels)| {
            let concrete_ty = render_type(&family.sort_template, &bindings);
            Leaf { concrete_ty, slot_labels, labels }
        })
        .collect()
}

// ============================================================
// Code generation
// ============================================================

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn gen_combination(family: &SortFamilyInput, leaf: &Leaf, idx: usize) -> TokenStream2 {
    let concrete_ty = &leaf.concrete_ty;

    // Sort name: base + "<label, label, ...>" for non-empty labels
    let non_empty: Vec<&str> =
        leaf.labels.iter().filter(|l| !l.is_empty()).map(|l| l.as_str()).collect();
    let sort_name = if non_empty.is_empty() {
        family.name.clone()
    } else {
        format!("{}<{}>", family.name, non_empty.join(", "))
    };

    // Navigation path substitution rules:
    // - `{variant}`: all non-empty labels joined with " + ", or "classic" if none.
    // - `{SlotName}`: that slot's label, omitted if empty.
    // - literal string: kept as-is.
    let path_elems: Vec<String> = family
        .path_template
        .iter()
        .filter_map(|p| {
            if p == "{variant}" {
                let joined = non_empty.join(" + ");
                Some(if joined.is_empty() { "classic".to_string() } else { joined })
            } else if let Some(inner) = p.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                let label = leaf.slot_labels.get(inner).map(|s| s.as_str()).unwrap_or("");
                if label.is_empty() { None } else { Some(label.to_string()) }
            } else {
                Some(p.clone())
            }
        })
        .collect();

    let stable = family.stable;
    let big_o = &family.big_o;

    // Unique identifier prefix: __sf_<family>_<idx>_<labels>
    let fam_s = sanitize(&family.name);
    let lbl_s = if non_empty.is_empty() {
        "base".to_string()
    } else {
        non_empty.iter().map(|l| sanitize(l)).collect::<Vec<_>>().join("_")
    };

    let fn_sort     = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_sort_fn");
    let fn_bench    = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_bench");
    let fn_register = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_register");
    let st_bench    = format_ident!(
        "__SF_{}_{}_{}_BENCH",
        fam_s.to_uppercase(),
        idx,
        lbl_s.to_uppercase()
    );

    // Build the call expression for sort_fn (NoOpLogger).
    let sort_call_noop = if family.direct_sort {
        quote! { <#concrete_ty>::sort(arr, logger) }
    } else {
        quote! {
            <#concrete_ty as crate::traits::sort_traits::SortAlgo<
                usize,
                crate::traits::log_traits::NoOpLogger,
            >>::sort(arr, logger)
        }
    };

    let st_test_mod = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_test");
    let st_cap      = format_ident!(
        "__SF_{}_{}_{}_CAP",
        fam_s.to_uppercase(),
        idx,
        lbl_s.to_uppercase()
    );

    let cap_static = match family.max_n_for_tests {
        Some(n) => {
            let lit = proc_macro2::Literal::u64_unsuffixed(n);
            quote! {
                #[linkme::distributed_slice(crate::bench_registry::SORT_TEST_CAPS)]
                #[allow(non_upper_case_globals)]
                static #st_cap: (&'static str, usize) = (#sort_name, #lit as usize);
            }
        }
        None => quote! {},
    };

    let base = quote! {
        #cap_static
        #[allow(non_snake_case, dead_code)]
        fn #fn_sort(
            arr: &mut [usize],
            logger: &mut crate::traits::log_traits::NoOpLogger,
        ) {
            #sort_call_noop;
        }

        #[allow(non_snake_case, dead_code)]
        fn #fn_bench(arr: &mut [usize]) {
            let mut l = crate::traits::log_traits::NoOpLogger;
            #fn_sort(arr, &mut l);
        }

        #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
        #[allow(non_upper_case_globals)]
        static #st_bench: crate::bench_registry::SortBenchEntry =
            crate::bench_registry::SortBenchEntry {
                name: #sort_name,
                big_o: #big_o,
                stable: #stable,
                run: #fn_bench,
            };

        #[cfg(test)]
        #[allow(non_snake_case)]
        mod #st_test_mod {
            #[test]
            fn correctness() {
                crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                    &super::#st_bench,
                    crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                );
            }
        }
    };

    if family.direct_sort {
        // Also generate sort_vis and register in SORT_VIS_REGISTRY.
        let fn_vis      = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_sort_vis");

        quote! {
            #base

            #[allow(non_snake_case, dead_code)]
            fn #fn_vis(
                arr: &mut [usize],
                logger: &mut dyn crate::traits::log_traits::SortLogger<usize>,
            ) {
                <#concrete_ty>::sort(arr, logger);
            }

            #[ctor::ctor]
            #[allow(non_snake_case)]
            fn #fn_register() {
                crate::traits::SORT_REGISTRY
                    .lock()
                    .unwrap()
                    .insert(#sort_name.to_string(), #fn_sort);
                crate::traits::SORT_VIS_REGISTRY
                    .lock()
                    .unwrap()
                    .insert(#sort_name.to_string(), #fn_vis);
                sort_registry_core::register_sort_path(
                    #sort_name,
                    #big_o,
                    #stable,
                    &[#(#path_elems),*],
                );
            }
        }
    } else {
        quote! {
            #base

            #[ctor::ctor]
            #[allow(non_snake_case)]
            fn #fn_register() {
                crate::traits::SORT_REGISTRY
                    .lock()
                    .unwrap()
                    .insert(#sort_name.to_string(), #fn_sort);
                sort_registry_core::register_sort_path(
                    #sort_name,
                    #big_o,
                    #stable,
                    &[#(#path_elems),*],
                );
            }
        }
    }
}

// ============================================================
// Entry point
// ============================================================

pub(crate) fn expand(family: SortFamilyInput) -> TokenStream2 {
    let leaves = enumerate(&family);
    if leaves.is_empty() {
        return syn::Error::new(Span::call_site(), "sort_family!: no combinations generated")
            .to_compile_error();
    }
    let items: Vec<_> = leaves
        .iter()
        .enumerate()
        .map(|(i, leaf)| gen_combination(&family, leaf, i))
        .collect();
    quote! { #(#items)* }
}
