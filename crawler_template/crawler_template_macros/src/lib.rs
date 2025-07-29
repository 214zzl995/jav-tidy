use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Crawler)]
pub fn derive_crawler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = input.data
    {
        &fields.named
    } else {
        panic!("Crawler only supports structs with named fields");
    };

    let crawler_path = quote! { ::crawler_template };

    let field_initializers = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_str = field_name.as_ref().unwrap().to_string();
        let field_type = &f.ty;

        let conversion_logic = match analyze_field_type(field_type) {
            FieldType::Direct => quote! {
                match map.get(#field_str).and_then(|v| v.first()) {
                    Some(s) => <#field_type as std::str::FromStr>::from_str(s)
                        .map_err(|_| #crawler_path::CrawlerParseError::ConversionFailed(#field_str))?,
                    None => return Err(#crawler_path::CrawlerParseError::MissingField(#field_str)),
                }
            },
            FieldType::OptionDirect => {
                // Extract the inner type from Option<T>
                let inner_type = extract_type_from_option(field_type);
                quote! {
                    map.get(#field_str)
                        .and_then(|v| v.first())
                        .map(|s| <#inner_type as std::str::FromStr>::from_str(s))
                        .transpose()
                        .map_err(|_| #crawler_path::CrawlerParseError::ConversionFailed(#field_str))?
                }
            },
            FieldType::VecDirect => {
                // Extract the inner type from Vec<T>
                let inner_type = extract_type_from_vec(field_type);
                quote! {
                    map.get(#field_str)
                        .map(|values| {
                            values.iter()
                                .map(|s| <#inner_type as std::str::FromStr>::from_str(s))
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .unwrap_or(Ok(Vec::new()))
                        .map_err(|_| #crawler_path::CrawlerParseError::ConversionFailed(#field_str))?
                }
            },
            FieldType::OptionVec => {
                // Extract the inner type from Option<Vec<T>>
                let inner_type = extract_type_from_option_vec(field_type);
                quote! {
                    map.get(#field_str)
                        .map(|values| {
                            if values.is_empty() {
                                Ok(None)
                            } else {
                                values.iter()
                                    .map(|s| <#inner_type as std::str::FromStr>::from_str(s))
                                    .collect::<Result<Vec<_>, _>>()
                                    .map(Some)
                            }
                        })
                        .transpose()
                        .map_err(|_| #crawler_path::CrawlerParseError::ConversionFailed(#field_str))?
                        .flatten()
                }
            },
        };

        quote! { #field_name: #conversion_logic }
    });

    let expanded = quote! {
        impl #crawler_path::CrawlerData for #struct_name {
            type Error = #crawler_path::CrawlerParseError;

            fn parse(map: &std::collections::HashMap<String, Vec<String>>) -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#field_initializers,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

// Type analysis logic
enum FieldType {
    Direct,       // T
    OptionDirect, // Option<T>
    VecDirect,    // Vec<T>
    OptionVec,    // Option<Vec<T>>
}

fn analyze_field_type(ty: &syn::Type) -> FieldType {
    let type_string = ty.to_token_stream().to_string();
    if type_string.starts_with("Option < Vec <") {
        FieldType::OptionVec
    } else if type_string.starts_with("Vec <") {
        FieldType::VecDirect
    } else if type_string.starts_with("Option <") {
        FieldType::OptionDirect
    } else {
        FieldType::Direct
    }
}

// Helper functions to extract inner types
fn extract_type_from_option(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return inner_type.clone();
                    }
                }
            }
        }
    }
    panic!("Could not extract inner type from Option");
}

fn extract_type_from_vec(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return inner_type.clone();
                    }
                }
            }
        }
    }
    panic!("Could not extract inner type from Vec");
}

fn extract_type_from_option_vec(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        if let syn::Type::Path(inner_path) = inner_type {
                            if let Some(inner_segment) = inner_path.path.segments.first() {
                                if inner_segment.ident == "Vec" {
                                    if let syn::PathArguments::AngleBracketed(inner_args) =
                                        &inner_segment.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(vec_inner_type)) =
                                            inner_args.args.first()
                                        {
                                            return vec_inner_type.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("Could not extract inner type from Option<Vec<T>>");
}
