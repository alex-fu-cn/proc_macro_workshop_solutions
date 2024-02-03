use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use syn::{
    DeriveInput, Expr, Field, GenericArgument, GenericParam, Ident, Meta, PathArguments, Type,
    TypePath,
};

// Debug-05 added `DebugMacroHelper`
pub struct DebugMacroHelper<'a> {
    pub generic_types: Vec<&'a Ident>,
    pub field_helpers: Vec<DebugMacroFieldHelper>,
}

// Debug-05 changed `DebugMacroFieldHelper` name and added a few fields.
pub struct DebugMacroFieldHelper {
    field_ident: Ident,
    field_type: Type,
    debug_format: Option<String>,
    is_phantom_data: bool,
    generic_param: Option<Type>,
}

impl<'a> DebugMacroHelper<'a> {
    pub fn new(ast: &'a DeriveInput) -> Result<Self, proc_macro::TokenStream> {
        let generic_types = extract_generic_types(&ast);
        let field_helpers = init_field_helpers(&ast);
        Ok(Self {
            generic_types,
            field_helpers,
        })
    }

    // Return generic types not contained by PhantomData.
    pub fn non_debug_types(&self) -> Vec<String> {
        let helpers = &self.field_helpers;
        let generic_types_string = self
            .generic_types
            .iter()
            .map(|t| t.to_token_stream().to_string())
            .collect::<Vec<String>>();
        // println!("GT: {:?}", generic_types_string);
        let non_debug_helpers = helpers.iter().filter(|h| {
            !h.is_phantom_data
                && generic_types_string.contains(&h.field_type.to_token_stream().to_string())
        });
        // println!("NDH: {:?}", non_debug_helpers);
        let result = non_debug_helpers
            .map(|h| h.field_type.to_token_stream().to_string())
            .collect::<Vec<String>>();
        // println!("Result: {:?}", result);
        result
    }

    // Add `T: Debug`
    pub fn to_required_debug_where_clause(&self) -> Vec<TokenStream> {
        let non_debug_types = self.non_debug_types();
        let pieces = non_debug_types
            .iter()
            .map(|t| {
                let ty = format_ident!("{}", t);
                quote! { #ty: std::fmt::Debug }
            })
            .collect::<Vec<TokenStream>>();
        pieces
    }
}

impl<'a> std::fmt::Debug for DebugMacroHelper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generic_type_debug = self
            .generic_types
            .iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let helper_debug = self
            .field_helpers
            .iter()
            .map(|h| format!("{:?}", h))
            .collect::<Vec<String>>()
            .join(",");
        f.debug_struct("DebugMacroHelper")
            .field("generic_types", &format_args!("{}", generic_type_debug))
            .field("field_helpers", &format_args!("{}", helper_debug))
            .finish()
    }
}

impl DebugMacroFieldHelper {
    // Return: ".field(#field, format_args!(#exp, #value))"
    pub fn to_debug_inner_form(&self) -> proc_macro2::TokenStream {
        let default = String::from("{:?}");
        let debug_format = self.debug_format.as_ref().unwrap_or(&default);
        let ident = &self.field_ident;
        let field_name = ident.to_string();
        if is_numeric_type(&self.field_type) {
            quote! {
                .field(#field_name, &format_args!(#debug_format, self.#ident))
            }
        } else {
            let quoted = format!("{}", debug_format);
            quote! {
                .field(#field_name, &format_args!(#quoted, self.#ident))
            }
        }
    }
}

impl std::fmt::Debug for DebugMacroFieldHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generic_param = &self.generic_param.to_token_stream().to_string();
        f.debug_struct("DebugMacroFieldHelper")
            .field("field_ident", &self.field_ident.to_string())
            .field("field_type", &self.field_type)
            .field("debug_format", &self.debug_format)
            .field("is_phantom_data", &self.is_phantom_data)
            .field("generic_param", generic_param)
            .finish()
    }
}

// Initialize helpers by parsed input.
fn init_field_helpers<'a>(ast: &DeriveInput) -> Vec<DebugMacroFieldHelper> {
    let mut helpers = Vec::new();
    if let syn::Data::Struct(data_struct) = &ast.data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let debug_format = extract_meta_name_value(&field, "debug");
                let generic_param = extract_generic_param(&field.ty);
                let helper = DebugMacroFieldHelper {
                    field_ident: field.ident.clone().unwrap(),
                    field_type: field.ty.clone(),
                    debug_format,
                    is_phantom_data: is_phantom_data(&field.ty),
                    generic_param,
                };
                helpers.push(helper);
            }
        }
    }
    helpers
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

fn extract_generic_types(ast: &DeriveInput) -> Vec<&Ident> {
    // println!("====================================");
    let mut generic_types = Vec::new();
    for param in &ast.generics.params {
        if let GenericParam::Type(tp) = param {
            // println!("{:#?}", &tp);
            generic_types.push(&tp.ident);
        }
    }
    generic_types
}

// Extract T from Option<T>, Vec<T>, etc.
fn extract_generic_param(ty: &Type) -> std::option::Option<Type> {
    match ty {
        Type::Path(ty_path) => {
            for segment in &ty_path.path.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let std::option::Option::Some(GenericArgument::Type(arg)) = args.args.first()
                    {
                        return extract_generic_param(arg)
                            .or(std::option::Option::Some(arg.clone()));
                    }
                }
            }
        }
        _ => {}
    }
    // Not a generic type
    None
}

fn is_phantom_data(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        for seq in &tp.path.segments {
            if !seq.arguments.is_none() && seq.ident == "PhantomData" {
                return true;
            }
        }
    }
    false
}
