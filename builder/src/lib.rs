use proc_macro::TokenStream;
use quote::quote;

// Key 1. #[proc_macro_derive(???)], set right derive name: Builder
// Key 2. Import TokenStream
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    // Key 3. Return TokenStream by calling quote!{}.into()
    let expanded = quote! {};
    expanded.into()
}
