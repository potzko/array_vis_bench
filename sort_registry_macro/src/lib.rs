use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use heck::ToSnakeCase;

/// Derive macro for automatically registering sorts in the global registry.
///
/// Intended for generic sort types that implement `crate::traits::sort_traits::SortAlgo<T, U>`.
/// When derived, a `crate::traits::SortRegistry` implementation is generated that:
/// - Registers a function pointer in `crate::traits::SORT_REGISTRY` keyed by the sort's `name()`
/// - Adds the sort's name to `crate::traits::SORT_NAMES` via `register_sort(name, big_o, stable, _)`
/// - Uses `#[ctor::ctor]` to ensure registration occurs at program startup
///
/// Function pointers allow full compile-time monomorphization and inlining (no trait objects).
///
/// Usage (with `create_sort!`): the macro is applied to the monomorphic `SortReg` type that `create_sort!` generates.
#[proc_macro_derive(SortRegistry)]
pub fn derive_sort_registry(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Unique function name for registration (snake_case to satisfy lint)
    let snake = name.to_string().to_snake_case();
    let register_fn = syn::Ident::new(&format!("__register_{}", snake), name.span());
    
    // Function pointer name for the monomorphic sort implementation
    let sort_fn_name = syn::Ident::new(&format!("__sort_fn_{}", snake), name.span());

    let expanded = quote! {
        /// Monomorphic sort function - compiled with full optimizations for usize/NoOpLogger
        fn #sort_fn_name(arr: &mut [usize], logger: &mut crate::traits::log_traits::NoOpLogger) {
            <#name as crate::traits::sort_traits::SortAlgo<usize, crate::traits::log_traits::NoOpLogger>>::sort(arr, logger);
        }

        impl #impl_generics sort_registry_core::SortRegistry for #name #ty_generics #where_clause {
            fn register() {
                use std::sync::Once;
                static REGISTERED: Once = Once::new();
                REGISTERED.call_once(|| {
                    // Resolve metadata via SortAlgo for concrete usize/NoOpLogger
                    type TReg = usize;
                    type UReg = crate::traits::log_traits::NoOpLogger;
                    let sort_name: &'static str = <#name as crate::traits::sort_traits::SortAlgo<TReg, UReg>>::name();
                    let big_o: &'static str = <#name as crate::traits::sort_traits::SortAlgo<TReg, UReg>>::big_o();
                    let stable: bool = <#name as crate::traits::sort_traits::SortAlgo<TReg, UReg>>::stable();

                    // Register function pointer (fully inlinable, no trait object overhead)
                    crate::traits::SORT_REGISTRY
                        .lock()
                        .unwrap()
                        .insert(
                            sort_name.to_string(),
                            #sort_fn_name as crate::traits::SortFn
                        );

                    // Also register metadata into names list (core)
                    sort_registry_core::register_sort(sort_name, big_o, stable, module_path!());
                });
            }
        }

        #[ctor::ctor]
        fn #register_fn() {
            <#name #ty_generics as sort_registry_core::SortRegistry>::register();
        }
    };
    TokenStream::from(expanded)
}
