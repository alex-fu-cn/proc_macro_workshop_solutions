use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod utils;
use utils::*;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // Pretty print DeriveInput if syn extra-traits enabled.
    // println!("{:#?}", ast);

    let derive_struct_ident = ast.ident;
    let builder_struct_ident = format_ident!("{}Builder", derive_struct_ident);

    // the structure data of the target struct
    let struct_data = ast.data;

    // Initializing macro helpers.
    let helpers = init_field_macro_helpers(&struct_data);

    // Key 1.1 Generate fields dynamically, check field_defintion_inner_form().
    let builder_field_definition_block = helpers.iter().map(|h| h.field_defintion_inner_form());
    // Key 1.2 Adding fields to Builder, google #(#var)*.
    let builder_definition_block = quote! {
        pub struct #builder_struct_ident {
            // field_name: Option<field_type>, ...
            #(#builder_field_definition_block)*
        }
    };

    // Key 2.1 Generate field init code for builder() function.
    let builder_constructor_inner = helpers.iter().map(|h| h.field_construction_inner_form());
    // Key 2.2 Builder constructor must initialize all fields correctly.
    let builder_constructor_block = quote! {
        impl #derive_struct_ident {
            pub fn builder() -> #builder_struct_ident {
                #builder_struct_ident {
                    // field_name: None, ...
                    #(#builder_constructor_inner)*
                }
            }
        }
    };

    // Key 3. Builder must have all setter functions implemented.
    let builder_field_setter = helpers
        .iter()
        .map(|h: &BuilderMacroFieldHelper<'_>| h.field_setter_from());
    let builder_implementation_block = quote! {
        impl #builder_struct_ident {
            #(#builder_field_setter)*
        }
    };

    // Populate the code blocks.
    let expanded = quote! {
        #builder_definition_block

        #builder_implementation_block

        #builder_constructor_block
    };
    expanded.into()
}
