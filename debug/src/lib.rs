use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, WherePredicate};

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

    // Key 1. Get generic variables
    let (impl_generics, ty_generics, where_clause) = struct_generic.split_for_impl();

    // Key 2. Process, there will be four situations
    //  a. Derive struct does not have generic type
    //  b. Derive struct has generic types but it doesn't have where clause
    //  c. Derive struct has generic types and it has where clause
    //  d. Derive struct has generic types and it has Debug & Display in where clause (ignore, leave it to compiler.)

    let is_generic_struct = ast.generics.params.len() > 0;
    let debug_where_clause = if is_generic_struct {
        let debug_predicate: WherePredicate = parse_quote!(T: std::fmt::Debug + std::fmt::Display);
        let new_where_clause = if where_clause.is_none() {
            // b
            quote! {
                where #debug_predicate
            }
        } else {
            // c
            quote! {
                #where_clause
                #debug_predicate
            }
        };
        new_where_clause
    } else {
        // a
        quote! {}
    };

    // Key 3. Populate code block correctly.
    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #derive_struct_ident #ty_generics #debug_where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                f.debug_struct("Field")
                    #(#field_debug_inner)*
                    .finish()
            }
        }
    };

    expanded.into()
}
