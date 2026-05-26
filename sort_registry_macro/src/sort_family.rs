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
    big_o: BigOValue,
    space: SpaceValue,
    stable: bool,
    /// True when the user wrote `stable = inherited`. In that case `stable`
    /// itself is ignored at emit time — the field is pulled from the type's
    /// `HasStability` impl instead.
    stable_inherited: bool,
    adaptive: bool,
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
    /// When true: emit a `#[inline(never)] #[no_mangle] pub fn asm_<slug>(
    /// arr: &mut [usize])` wrapper around the NoOp-logger sort call for each
    /// variant, plus a `#[used]` static anchor so the linker doesn't drop the
    /// symbol. Off by default; flip on for the family you're investigating
    /// and pair with `cargo asm` for a clean ASM dump.
    asm_friendly: bool,
}

struct AxisDef {
    slot: String,
    variants: Vec<VariantNode>,
}

/// What `big_o = ...` resolved to during parsing.
///
/// `Literal("O(N log N)")` — legacy string form. Emitted as a const
/// `Complexity::from_str("O(N log N)")` expression, which is parsed at
/// compile time inside the AlgorithmEntry static initializer.
///
/// `Inherited` — emitted as `<ConcreteType as HasTimeBounds>::WORST`,
/// pulling the value from the per-axis composable annotation pipeline.
/// Use for sorts whose components declare their own HasTimeBounds.
pub(crate) enum BigOValue {
    Literal(String),
    Inherited,
}

/// What `space = ...` resolved to during parsing, plus a default for the
/// (currently common) case where no `space` field was given.
pub(crate) enum SpaceValue {
    /// `space = "O(...)"` — parsed at compile time by `Complexity::from_str`.
    Literal(String),
    /// `space = inherited` — pulls `<ConcreteType as HasSpace>::SPACE`.
    Inherited,
    /// No `space` field present. Defaults to `Complexity::CONST` at emit
    /// time (conservative "in-place" default for sorts that haven't been
    /// annotated yet).
    Default,
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
    /// `{Slot}` or `{Slot}<arg, …>` — the slot's bound type is wrapped in
    /// the trailing generic args (if any) at substitution time.
    Slot(String, Vec<TypeExprArg>),
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
    // A type expression either starts with `{` (bare slot, optionally
    // followed by generic args), a bool/int literal, or an Ident.
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let slot: Ident = content.parse()?;
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
        return Ok(TypeExpr::Slot(slot.to_string(), args));
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
        let content;
        braced!(content in input);
        let slot: Ident = content.parse()?;
        // Bare `{Slot}` becomes TypeExprArg::Slot; `{Slot}<args>` is
        // wrapped as TypeExprArg::Type(TypeExpr::Slot(name, args)) so the
        // trailing generics ride along through render_type.
        if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            let mut args = vec![];
            while !input.peek(Token![>]) {
                args.push(parse_type_expr_arg(input)?);
                if input.peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                }
            }
            let _: Token![>] = input.parse()?;
            return Ok(TypeExprArg::Type(TypeExpr::Slot(slot.to_string(), args)));
        }
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
        let mut big_o: Option<BigOValue> = None;
        let mut space: SpaceValue = SpaceValue::Default;
        let mut stable: Option<bool> = None;
        let mut stable_inherited: bool = false;
        let mut adaptive: bool = false;
        let mut path_template: Vec<String> = vec![];
        let mut direct_sort: bool = false;
        let mut max_n_for_tests: Option<u64> = None;
        let mut asm_friendly: bool = false;

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
                        // Accept either a string literal ("O(N log N)") for
                        // the legacy hand-annotated form, or the bare ident
                        // `inherited` to pull the value from the type's
                        // `HasTimeBounds` impl at compile time.
                        if input.peek(LitStr) {
                            big_o = Some(BigOValue::Literal(input.parse::<LitStr>()?.value()));
                        } else if input.peek(Ident) {
                            let kw: Ident = input.parse()?;
                            if kw == "inherited" {
                                big_o = Some(BigOValue::Inherited);
                            } else {
                                return Err(syn::Error::new(
                                    kw.span(),
                                    format!("Expected `\"O(...)\"` literal or `inherited`, got `{kw}`"),
                                ));
                            }
                        } else {
                            return Err(input.error(
                                "Expected `big_o = \"O(...)\"` or `big_o = inherited`",
                            ));
                        }
                        let _: Token![;] = input.parse()?;
                    }
                    "stable" => {
                        // `stable = true|false` (literal) or `stable = inherited`
                        // (pulled from `<T as HasStability>::STABLE`).
                        if input.peek(LitBool) {
                            stable = Some(input.parse::<LitBool>()?.value());
                        } else if input.peek(Ident) {
                            let kw: Ident = input.parse()?;
                            if kw == "inherited" {
                                stable_inherited = true;
                                stable = Some(false); // placeholder; not used when inherited
                            } else {
                                return Err(syn::Error::new(
                                    kw.span(),
                                    format!("Expected `true`, `false`, or `inherited`, got `{kw}`"),
                                ));
                            }
                        } else {
                            return Err(input.error(
                                "Expected `stable = true|false` or `stable = inherited`",
                            ));
                        }
                        let _: Token![;] = input.parse()?;
                    }
                    "space" => {
                        // `space = "O(...)"` (legacy literal) or `space = inherited`
                        // (pulled from `<T as HasSpace>::SPACE`).
                        if input.peek(LitStr) {
                            space = SpaceValue::Literal(input.parse::<LitStr>()?.value());
                        } else if input.peek(Ident) {
                            let kw: Ident = input.parse()?;
                            if kw == "inherited" {
                                space = SpaceValue::Inherited;
                            } else {
                                return Err(syn::Error::new(
                                    kw.span(),
                                    format!("Expected `\"O(...)\"` literal or `inherited`, got `{kw}`"),
                                ));
                            }
                        } else {
                            return Err(input.error(
                                "Expected `space = \"O(...)\"` or `space = inherited`",
                            ));
                        }
                        let _: Token![;] = input.parse()?;
                    }
                    "adaptive" => {
                        adaptive = input.parse::<LitBool>()?.value();
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
                    "asm" => {
                        asm_friendly = input.parse::<LitBool>()?.value();
                        let _: Token![;] = input.parse()?;
                    }
                    other => {
                        return Err(syn::Error::new(
                            field.span(),
                            format!(
                                "Unknown field `{other}`. Expected: name, big_o, space, stable, adaptive, path, direct_sort, max_n_for_tests, asm"
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
            space,
            stable: stable.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "sort_family!: missing `stable = ...;`")
            })?,
            stable_inherited,
            adaptive,
            path_template,
            direct_sort,
            max_n_for_tests,
            asm_friendly,
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
        TypeExpr::Slot(s, args) => {
            let base = bindings
                .get(s)
                .cloned()
                .unwrap_or_else(|| quote! { compile_error!("sort_family!: unresolved slot") });
            if args.is_empty() {
                base
            } else {
                let rendered: Vec<_> = args.iter().map(|a| render_arg(a, bindings)).collect();
                quote! { #base < #(#rendered),* > }
            }
        }
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
    let adaptive = family.adaptive;
    // Build the AlgorithmEntry field expressions for this combination.
    //
    // - `big_o = "O(...)"` (legacy literal): parsed by `Complexity::from_str`
    //   at compile time. worst/best/average all get the same value.
    // - `big_o = inherited`: pulls WORST/BEST/AVERAGE individually from
    //   `<ConcreteType as HasTimeBounds>` so a sort with N² worst but
    //   N log N best (e.g. QuickSort with degenerate pivot) surfaces the
    //   distinction in the registry.
    let (worst_expr, best_expr, average_expr): (TokenStream2, TokenStream2, TokenStream2) =
        match &family.big_o {
            BigOValue::Literal(s) => {
                let parsed = quote! { ::array_vis_bench_traits::Complexity::from_str(#s) };
                (parsed.clone(), parsed.clone(), parsed)
            }
            BigOValue::Inherited => (
                quote! { <#concrete_ty as ::array_vis_bench_traits::composable::HasTimeBounds>::WORST },
                quote! { <#concrete_ty as ::array_vis_bench_traits::composable::HasTimeBounds>::BEST },
                quote! { <#concrete_ty as ::array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE },
            ),
        };
    // Space field expression. `space = "O(...)"` literal | `space = inherited`
    // | omitted → `Complexity::CONST` (conservative "in-place" default; most
    // sorts in the codebase are).
    let space_expr: TokenStream2 = match &family.space {
        SpaceValue::Literal(s) => quote! { ::array_vis_bench_traits::Complexity::from_str(#s) },
        SpaceValue::Inherited => quote! {
            <#concrete_ty as ::array_vis_bench_traits::composable::HasSpace>::SPACE
        },
        SpaceValue::Default => quote! { ::array_vis_bench_traits::Complexity::CONST },
    };
    // Stability: literal bool | `inherited` → HasStability::STABLE.
    let stable_expr: TokenStream2 = if family.stable_inherited {
        quote! { <#concrete_ty as ::array_vis_bench_traits::composable::HasStability>::STABLE }
    } else {
        quote! { #stable }
    };
    // Display form used at the legacy register_sort_path call (string-typed
    // and currently ignored downstream — kept for API stability).
    let big_o_display: TokenStream2 = match &family.big_o {
        BigOValue::Literal(s) => quote! { #s },
        BigOValue::Inherited => quote! {
            <#concrete_ty as ::array_vis_bench_traits::composable::HasTimeBounds>::WORST.as_str()
        },
    };

    // Unique identifier prefix: __sf_<family>_<idx>_<labels>
    let fam_s = sanitize(&family.name);
    let lbl_s = if non_empty.is_empty() {
        "base".to_string()
    } else {
        non_empty.iter().map(|l| sanitize(l)).collect::<Vec<_>>().join("_")
    };

    // Identifiers for the per-leaf helpers we emit. `__sf_<family>_<idx>
    // _<labels>_*` keeps them unique without colliding with anything else
    // the user might have in the same module.
    let fn_sort        = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_sort_fn");
    let fn_run_default = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_run_default");
    let fn_run_correct = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_run_correctness");
    let fn_register    = format_ident!("__sf_{fam_s}_{idx}_{lbl_s}_register");
    let st_entry       = format_ident!(
        "__SF_{}_{}_{}_ENTRY",
        fam_s.to_uppercase(),
        idx,
        lbl_s.to_uppercase()
    );
    let st_cap         = format_ident!(
        "__SF_{}_{}_{}_CAP",
        fam_s.to_uppercase(),
        idx,
        lbl_s.to_uppercase()
    );

    // Inherent method (direct_sort = true) vs trait route (direct_sort
    // = false). Both pass through to `Sort::sort(arr, logger)` —
    // `logger` is monomorphic-NoOp in the test path and a dyn
    // SortLogger in the visualisation path.
    let sort_call_noop = if family.direct_sort {
        quote! { <#concrete_ty>::sort(arr, logger) }
    } else {
        quote! {
            <#concrete_ty as ::array_vis_bench_traits::sort_traits::SortAlgo<
                usize,
                ::sort_logger::NoOpLogger,
            >>::sort(arr, logger)
        }
    };
    let sort_call_dyn = if family.direct_sort {
        quote! { <#concrete_ty>::sort(arr, logger) }
    } else {
        // SortAlgo<T, U> needs a sized U; visualisation for indirect
        // sorts isn't wired (none exist that need it). Fall back to a
        // no-op so the macro still compiles for those.
        quote! { let _ = (arr, logger); }
    };

    let cap_static = match family.max_n_for_tests {
        Some(n) => {
            let lit = proc_macro2::Literal::u64_unsuffixed(n);
            quote! {
                #[linkme::distributed_slice(::array_vis_bench_core::bench_registry::SORT_TEST_CAPS)]
                #[allow(non_upper_case_globals)]
                static #st_cap: (&'static str, usize) = (#sort_name, #lit as usize);
            }
        }
        None => quote! {},
    };

    // `asm = true` opt-in: emit a no-mangle wrapper so `cargo asm` can pull
    // a clean, deterministic dump for this variant. `#[inline(never)]`
    // forces a real call boundary even when the sort body would otherwise
    // be inlined into the test harness; `#[used]` on a fn-pointer static
    // keeps the linker from gc-ing the symbol when nothing in the crate
    // graph references `asm_<slug>` directly.
    let asm_extras = if family.asm_friendly {
        let asm_slug = format!("{fam_s}_{idx}_{lbl_s}");
        let fn_asm = format_ident!("asm_{asm_slug}");
        let st_anchor = format_ident!("__ASM_ANCHOR_{}", asm_slug.to_uppercase());
        quote! {
            #[inline(never)]
            #[no_mangle]
            #[allow(non_snake_case, dead_code)]
            pub fn #fn_asm(arr: &mut [usize]) {
                let logger = &mut ::sort_logger::NoOpLogger;
                #sort_call_noop;
            }
            #[used]
            #[allow(non_upper_case_globals, dead_code)]
            static #st_anchor: fn(&mut [usize]) = #fn_asm;
        }
    } else {
        quote! {}
    };

    quote! {
        #cap_static
        #asm_extras

        // Type-erased entry used by the correctness battery
        // (`sort_battery` takes a fn pointer with this exact shape).
        #[allow(non_snake_case, dead_code)]
        fn #fn_sort(
            arr: &mut [usize],
            logger: &mut ::sort_logger::NoOpLogger,
        ) {
            #sort_call_noop;
        }

        // Vis-side entry. Looks up the named input in `SORT_INPUTS`,
        // generates its values, emits the initial-state events on the
        // logger, then runs the sort with the supplied dyn logger.
        // All three responsibilities live inside the shared helper
        // `run_sort_with_input` to keep this macro-emitted body small.
        #[allow(non_snake_case, dead_code)]
        fn #fn_run_default(
            input_name: &str,
            config: &::array_vis_bench_core::bench_registry::RunConfig,
            logger: &mut dyn ::sort_logger::SortLogger<usize>,
        ) {
            fn sort_dyn(
                arr: &mut [usize],
                logger: &mut dyn ::sort_logger::SortLogger<usize>,
            ) {
                #sort_call_dyn;
            }
            ::array_vis_bench_core::bench_registry::run_sort_with_input(input_name, config, sort_dyn, logger);
        }

        // Test-side entry. Runs the shared sort battery + stability
        // battery (the latter is a no-op for non-stable sorts).
        #[allow(non_snake_case, dead_code)]
        fn #fn_run_correct() {
            ::array_vis_bench_core::bench_registry::correctness::sort_battery(#fn_sort, #sort_name);
            ::array_vis_bench_core::bench_registry::correctness::sort_stability_battery(
                #fn_sort, #sort_name, #stable_expr,
            );
        }

        #[linkme::distributed_slice(::array_vis_bench_core::bench_registry::ALGORITHMS)]
        #[allow(non_upper_case_globals)]
        static #st_entry: ::array_vis_bench_core::bench_registry::AlgorithmEntry =
            ::array_vis_bench_core::bench_registry::AlgorithmEntry {
                name: #sort_name,
                category: ::array_vis_bench_core::bench_registry::Category::Sort,
                worst: #worst_expr,
                best: #best_expr,
                average: #average_expr,
                space: #space_expr,
                stable: #stable_expr,
                adaptive: #adaptive,
                max_input_size: None,
                run_with_input: #fn_run_default,
                run_correctness: #fn_run_correct,
            };

        // Per-leaf subprocess test removed: the aggregate
        // `all_registered_algorithms_are_correct` in the wiring crate
        // already exercises every variant. Re-emitting it here would
        // duplicate work and force every leaf to depend on main's
        // `test_helpers` module.

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #fn_register() {
            // Every sort lives under the top-level "sorts" group so the
            // category picker — sorts / rotations / partitions / merges /
            // small-sorts — falls out of tree shape rather than living in
            // a separate piece of code. Rotations / partitions / small-sorts
            // register their own category prefix at their respective sites.
            sort_registry_core::register_sort_path(
                #sort_name,
                #big_o_display,
                #stable,
                &["sorts", #(#path_elems),*],
            );
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
