use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod utils;
use utils::*;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // Pretty print DeriveInput if syn extra-traits enabled.
    // println!("{:#?}", ast);

    let derive_struct_ident = ast.ident;
    let builder_struct_ident = format_ident!("{}Builder", derive_struct_ident);
    let struct_data = ast.data;

    let helpers = match init_field_macro_helpers(&struct_data) {
        Ok(vec) => vec,
        Err(e) => {
            // Key 4. Error returned.
            return e;
        }
    };

    let builder_field_definition_block = helpers.iter().map(|h| h.field_defintion_inner_form());
    let builder_definition_block = quote! {
        pub struct #builder_struct_ident {
            // field_name: Option<field_type>, ...
            #(#builder_field_definition_block)*
        }
    };

    let builder_constructor_inner = helpers.iter().map(|h| h.field_construction_inner_form());
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

    let builder_field_setter = helpers
        .iter()
        .map(|h: &BuilderMacroFieldHelper<'_>| h.field_setter_from());
    let build_inner = helpers.iter().map(|h| h.field_build_inner_form());
    let builder_build_method = quote! {
        fn build(&self) -> Result<#derive_struct_ident, Box<dyn std::error::Error>> {
            Ok(#derive_struct_ident {
                #(#build_inner)*
            })
        }
    };

    let builder_field_each_setter = helpers
        .iter()
        .map(|h: &BuilderMacroFieldHelper<'_>| h.field_setter_each_form());

    let builder_implementation_block = quote! {
        impl #builder_struct_ident {

            #(#builder_field_setter)*

            #(#builder_field_each_setter)*

            #builder_build_method

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
