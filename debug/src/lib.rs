use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod utils;
use utils::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let derive_struct_ident = &ast.ident;

    // initialize helpers
    let helper = match DebugMacroHelper::new(&ast) {
        Ok(vec) => vec,
        Err(e) => {
            return e;
        }
    };

    let field_debug_inner = helper.field_helpers.iter().map(|f| f.to_debug_inner_form());

    // If derived struct has generic type.
    let struct_generic = &ast.generics;
    // println!("{:#?}", struct_generic);

    // In order to pass debug-05, all generic types that not contained by PhantomData must implemented Debug trait.
    // So we must have these key steps:
    // a. Get the generic type code fragement from source struct, because our new predicates shall be added to existed ones.
    let (impl_generics, ty_generics, where_clause) = struct_generic.split_for_impl();

    // b. Construct the where clause.
    //    1) If there is generic type that's not contained by PhantomData, add `T: Debug` to where clause.
    //    2) If the generic type is contained by PhantomData, leave it unchanged.
    let is_generic_struct = ast.generics.params.len() > 0;
    let where_clause_parts = helper.to_required_debug_where_clause();
    let debug_where_clause = if is_generic_struct {
        if where_clause.is_some() {
            let wc = where_clause.unwrap();
            quote! {
                #wc,
                #(#where_clause_parts),*
            }
        } else {
            quote! {
                where #(#where_clause_parts),*
            }
        }
    } else {
        quote! {}
    };

    // c. Change the implementation block accordingly.
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
