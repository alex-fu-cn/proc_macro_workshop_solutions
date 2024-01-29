use quote::quote;
use quote::{format_ident, ToTokens};
use syn::{Data, GenericArgument, PathArguments, Type};

// Helper struct, help generating code pieces.
pub struct BuilderMacroFieldHelper<'a> {
    field_name: String,
    field_type: &'a Type,
    is_option_type: bool,
    is_vec_type: bool,
}

// Implementation of the helper.
impl<'a> BuilderMacroFieldHelper<'a> {
    // code block in Builder struct:
    //     struct CommandBuilder {
    //         $$$$ <- generate these
    //     }
    pub fn field_defintion_inner_form(&self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.field_name);
        let ty = self.field_type;
        if self.is_option_type {
            quote! {
                #name: #ty, // Option<Option<T>> is not needed
            }
        } else {
            quote! {
                #name: Option<#ty>,
            }
        }
    }

    // code block in builder() function:
    //    fn builder() -> CommandBuilder {
    //         CommandBuilder {
    //             $$$$$ <- generate these
    //         }
    //    }
    pub fn field_construction_inner_form(&self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.field_name);
        if self.is_vec_type {
            quote! {
                #name: Some(Vec::new()),
            }
        } else {
            quote! {
                #name: None,
            }
        }
    }

    // code block of chained setter methods
    pub fn field_setter_from(&self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.field_name);
        let ty = self.field_type;
        if self.is_option_type {
            let inner_ty = extract_generic_type(ty);
            quote! {
                fn #name(&mut self, #name:#inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                fn #name(&mut self, #name:#ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    }

    // code block of build function:
    // fn build() -> Command {
    //     Command {
    //         $$$$$ <- generate these
    //     }
    // }
    pub fn field_build_inner_form(&self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.field_name);
        if self.is_option_type {
            quote! {
                #name: self.#name.clone(),
            }
        } else {
            quote! {
                #name: self.#name.clone().unwrap(),
            }
        }
    }
}

// Initialize helper vector.
pub fn init_field_macro_helpers(struct_data: &Data) -> Vec<BuilderMacroFieldHelper> {
    let mut helpers: Vec<BuilderMacroFieldHelper> = Vec::new();
    if let syn::Data::Struct(data_struct) = &struct_data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_name = field.ident.to_token_stream().to_string();
                helpers.push(BuilderMacroFieldHelper {
                    field_name: field_name,
                    field_type: &field.ty,
                    is_option_type: is_type_eq(&field.ty, "Option"),
                    is_vec_type: is_type_eq(&field.ty, "Vec"),
                });
            }
        }
    }
    helpers
}

// Check whether type (ty) is the specified type (tystr).
pub fn is_type_eq(ty: &Type, tystr: &str) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            return last_segment.ident == tystr;
        }
    }
    false
}

// Extract T from Option<T>, Vec<T>, etc.
fn extract_generic_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(ty_path) => {
            for segment in &ty_path.path.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(arg)) = args.args.first() {
                        return extract_generic_type(arg).or(Some(arg));
                    }
                }
            }
        }
        _ => {}
    }
    // Not a generic type
    None
}
