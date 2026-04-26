use proc_macro::TokenStream;
use syn::parse_macro_input;

mod sort_family;

/// Generate all concrete sort combinations from a declarative variant-tree description.
///
/// # Syntax
///
/// ```ignore
/// sort_family! {
///     type Sort = RootType<{Slot1}, {Slot2}>;
///
///     Slot1 {
///         ConcreteType1 => "label1"
///         GenericType<{SubSlot}> => "label2" {
///             SubSlot { SubTypeA => "a"  SubTypeB => "b" }
///         }
///     }
///
///     Slot2 { /* … */ }
///
///     name   = "human readable name";
///     big_o  = "O(N log N)";
///     stable = false;
///     path   = ["category", "{Slot1}", "{SubSlot}"];
/// }
/// ```
///
/// For each leaf in the variant tree one anonymous `fn` + `static` block is generated
/// that registers the combination in `BENCH_SORTS` (linkme) and `SORT_REGISTRY` (ctor).
#[proc_macro]
pub fn sort_family(input: TokenStream) -> TokenStream {
    let family = parse_macro_input!(input as sort_family::SortFamilyInput);
    TokenStream::from(sort_family::expand(family))
}
