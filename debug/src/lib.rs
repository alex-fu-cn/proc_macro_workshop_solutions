use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod utils;
use utils::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let derive_struct_ident = ast.ident;
    let struct_data = &ast.data;

    // initialize helpers
    let helpers = match init_debug_macro_helpers(struct_data) {
        Ok(vec) => vec,
        Err(e) => {
            return e;
        }
    };

    let field_debug_inner = helpers.iter().map(|f| f.to_debug_inner_form());

    // If derived struct has generic type.
    let struct_generic = &ast.generics;
    // println!("{:#?}", struct_generic);
    let is_generic_struct = struct_generic.params.len() > 0;
    // Key: Restriction: T must implement Debug
    let (impl_generics, ty_generics, raw_where_clause) = ast.generics.split_for_impl();
    let where_clause = if is_generic_struct {
        quote! {
            where
            #raw_where_clause
            T: std::fmt::Debug + std::fmt::Display
        }
    } else {
        quote! {
            where
            #raw_where_clause
        }
    };

    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #derive_struct_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                f.debug_struct("Field")
                    #(#field_debug_inner)*
                    .finish()
            }
        }
    };
    expanded.into()
}
