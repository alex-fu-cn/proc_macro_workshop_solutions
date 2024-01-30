use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod utils;
use utils::*;

// Key 1. "debug" attribute configured here.
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let derive_struct_ident = ast.ident;

    // initialize helpers
    let helpers = match init_debug_macro_helpers(&ast.data) {
        Ok(vec) => vec,
        Err(e) => {
            return e;
        }
    };

    // Key 1.1 generate fmt fn, the inner part: .field("field_name", field_value);
    let field_debug_inner = helpers.iter().map(|f| f.to_debug_inner_form());

    // Key 1.2 complete the fmt fn.
    let expanded = quote! {
        impl std::fmt::Debug for #derive_struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                f.debug_struct("Field")
                    #(#field_debug_inner)*
                    .finish()
            }
        }
    };
    expanded.into()
}
