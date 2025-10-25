use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam};

/// Derive macro for automatically registering sorts in the global registry
///
/// This macro will:
/// 1. Extract the type name and generic parameters
/// 2. Generate a registration function that adds the sort to the global registry
/// 3. Ensure the registration happens at compile time
///
/// Usage:
/// ```rust
/// #[derive(SortRegistry)]
/// pub struct MySort<T, U> {
///     // sort implementation
/// }
/// ```
#[proc_macro_derive(SortRegistry)]
pub fn derive_sort_registry(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate a unique function name for registration
    let register_fn = syn::Ident::new(&format!("__register_{}", name), name.span());

    // Extract generic parameter names for the registry
    let type_params: Vec<_> = generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(type_param) = param {
                Some(&type_param.ident)
            } else {
                None
            }
        })
        .collect();

    // Create a type name string that includes generic parameters
    let type_name = if type_params.is_empty() {
        quote! { stringify!(#name).to_string() }
    } else {
        let param_names: Vec<_> = type_params
            .iter()
            .map(|param| quote! { stringify!(#param) })
            .collect();
        quote! {
            {
                let mut name = stringify!(#name).to_string();
                name.push('<');
                name.push_str(&[#(#param_names),*].join(", "));
                name.push('>');
                name
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics crate::traits::SortRegistry for #name #ty_generics #where_clause {
            fn register() {
                use std::sync::Once;
                static REGISTERED: Once = Once::new();
                REGISTERED.call_once(|| {
                    let type_name = #type_name;
                    crate::traits::SORT_REGISTRY.lock().unwrap().insert(
                        type_name,
                        Box::new(|arr: &mut [i32], logger: &mut crate::traits::log_traits::NoOpLogger| {
                            Self::sort(arr, logger);
                        })
                    );
                });
            }

            fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
                // This will be implemented by the user
                // For now, we'll provide a default implementation that does nothing
                // The user should override this method
            }
        }

        #[ctor::ctor]
        fn #register_fn() {
            <#name #ty_generics as crate::traits::SortRegistry>::register();
        }
    };
    TokenStream::from(expanded)
}
