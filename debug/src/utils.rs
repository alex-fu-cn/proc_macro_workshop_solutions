use quote::{format_ident, quote, ToTokens};

use syn::{Data, Expr, Field, Meta, Type, TypePath};

pub struct DebugMacroFieldHelper<'a> {
    field_name: String,
    field_type: &'a Type,
    debug_format: Option<String>,
}

impl<'a> DebugMacroFieldHelper<'a> {
    // Return: ".field(#field, format_args!(#exp, #value))"
    pub fn to_debug_inner_form(&self) -> proc_macro2::TokenStream {
        let default = String::from("{}");
        let debug_format = self.debug_format.as_ref().unwrap_or(&default);
        let ident = format_ident!("{}", self.field_name);
        let field_name = &self.field_name;
        if is_numeric_type(self.field_type) {
            quote! {
                .field(#field_name, &format_args!(#debug_format, self.#ident))
            }
        } else {
            let quoted = format!("\"{}\"", debug_format);
            quote! {
                .field(#field_name, &format_args!(#quoted, self.#ident))
            }
        }
    }
}

// Initialize helpers by parsed input.
pub fn init_debug_macro_helpers(
    struct_data: &Data,
) -> Result<Vec<DebugMacroFieldHelper>, proc_macro::TokenStream> {
    let mut helpers: Vec<DebugMacroFieldHelper> = Vec::new();
    if let syn::Data::Struct(data_struct) = &struct_data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_name = field.ident.to_token_stream().to_string();
                // key 2. Extract the value of debug attribute.
                let debug_format = extract_meta_name_value(&field, "debug");
                helpers.push(DebugMacroFieldHelper {
                    field_name,
                    field_type: &field.ty,
                    debug_format,
                });
            }
        }
    }
    Ok(helpers)
}

// Check if the type is numeric or not.
fn is_numeric_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(last_segment) = path.segments.last() {
                match &last_segment.ident.to_string()[..] {
                    "i32" | "u32" | "f32" | "i64" | "u64" | "f64" | "i8" | "u8" | "i16" | "u16" => {
                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

// extract value, for example: #[debug = "value"], value will be returned (without quotes)
fn extract_meta_name_value(ty: &Field, ident: &str) -> Option<String> {
    for attr in &ty.attrs {
        if let Meta::NameValue(meta_name_value) = &attr.meta {
            if meta_name_value.path.is_ident(ident) {
                if let Expr::Lit(value) = &meta_name_value.value {
                    return std::option::Option::Some(trim_quotes(
                        value.to_token_stream().to_string(),
                    ));
                }
            }
        }
    }
    std::option::Option::None
}

// Trim quotes around
fn trim_quotes(quoted_string: String) -> String {
    if quoted_string.starts_with('\"') && quoted_string.ends_with('\"') {
        String::from(&quoted_string[1..quoted_string.len() - 1])
    } else {
        quoted_string
    }
}
