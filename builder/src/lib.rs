use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Key 2. Key parse function and DeriveInput struct.
    let ast = parse_macro_input!(input as DeriveInput);
    // Pretty print DeriveInput if syn extra-traits enabled.
    // println!("{:#?}", ast);

    // Key 3. Ident is not "String" and how to construct an ident.
    let derive_struct_ident = ast.ident;
    // It won't compile if this is String type, must be ident for struct name.
    let builder_struct_ident = format_ident!("{}Builder", derive_struct_ident);

    // Key 4.1 Get to know quote! macro. struct CommandBuilder.
    let builder_definition_block = quote! {
        // no fields needed for test 2
        pub struct #builder_struct_ident {}
    };

    // Key 5. Get to know quote! macro. Impl builder() function.
    let builder_constructor_block = quote! {
        impl #derive_struct_ident {
            pub fn builder() -> #builder_struct_ident {
                #builder_struct_ident {
                    // no fields need for test 2.
                }
            }
        }
    };

    // Key 1. Macro generated code is here. What we need to do is adding new code blocks.
    // Code blocks can be populated by other code blocks.
    let expanded = quote! {
        #builder_definition_block

        #builder_constructor_block
    };
    expanded.into()
}
