use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    // return TokenStream by calling quote::quote!
    let expanded = quote! {};
    expanded.into()
}
