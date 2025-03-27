use super::*;
use syn::{Data, DataStruct, Fields, FieldsNamed, punctuated::Punctuated, token::Comma, Field};
use quote::{quote, ToTokens};
use proc_macro2::TokenStream as TokenStream2;

pub fn expand_derive(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "crawler can only be applied to structs",
        ));
    };

    let matchs = build_try_set(named)?;

    let expanded = quote! {
        use ::crawler_template::CrawlerErr;

        impl ::crawler_template::CrawlerData for #ident {
            fn try_set(&mut self, field: &str, values: Vec<String>) -> Result<(), CrawlerErr> {
                match field {
                    #(#matchs)*
                    _ => Ok(()),
                }
            }
        }
    };

    Ok(expanded)
}

fn build_try_set(named: &Punctuated<Field, Comma>) -> syn::Result<Vec<TokenStream2>> {
    let mut matchs = Vec::new();

    for field in named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_attrs = &field.attrs;

        // Check for #[crawler(skip)] attribute
        let mut should_skip = false;
        for attr in field_attrs.iter() {
            if attr.path().is_ident("crawler") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        should_skip = true;
                        Ok(())
                    } else {
                        Err(meta.error("unrecognized attribute, only 'skip' is supported"))
                    }
                })?;
            }
        }
        if should_skip {
            continue;
        };

        if let syn::Type::Path(syn::TypePath { path, .. }) = field_ty {
            let segment = path.segments.last().unwrap();
            let ty_ident = &segment.ident;

            if ty_ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments { args, .. },
                ) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        if let syn::Type::Path(inner_type_path) = inner_ty {
                            let inner_ty_tokens = inner_type_path.to_token_stream();
                            matchs.push(quote! {
                                stringify!(#field_name) => {
                                    let vec: Vec<#inner_ty_tokens> = values
                                        .iter()
                                        .map(|v| v.parse::<#inner_ty_tokens>().map_err(|err| ::crawler_template::CrawlerErr::ParseError(stringify!(#field_name).to_string(),err.to_string())))
                                        .collect::<Result<Vec<_>, _>>()?;
                                    self.#field_name = vec;
                                    Ok(())
                                }
                            });
                        } else {
                            return Err(syn::Error::new_spanned(inner_ty, "unsupported inner type for Vec"));
                        }
                    }
                }
            } else if ty_ident == "Option" {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments { args, .. },
                ) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        if let syn::Type::Path(inner_type_path) = inner_ty {
                            let inner_ty_tokens = inner_type_path.to_token_stream();
                            matchs.push(quote! {
                                stringify!(#field_name) => {
                                    if values.len() > 1 {
                                        return Err(::crawler_template::CrawlerErr::InvalidValueCount(stringify!(#field_name).to_string(), values.len()));
                                    }
                                    self.#field_name = if let Some(v) = values.first() {
                                        Some(v.parse::<#inner_ty_tokens>().map_err(|err| ::crawler_template::CrawlerErr::ParseError(stringify!(#field_name).to_string(),err.to_string()))?)
                                    } else {
                                        None
                                    };
                                    Ok(())
                                }
                            });
                        } else {
                            return Err(syn::Error::new_spanned(inner_ty, "unsupported inner type for Option"));
                        }
                    }
                }
            } else {
                let ty_tokens = field_ty.to_token_stream();
                matchs.push(quote! {
                    stringify!(#field_name) => {
                        if values.len() != 1 {
                            return Err(::crawler_template::CrawlerErr::InvalidValueCount(stringify!(#field_name).to_string(), values.len()));
                        }
                        let value = values[0].parse::<#ty_tokens>().map_err(|err| ::crawler_template::CrawlerErr::ParseError(stringify!(#field_name).to_string(),err.to_string()))?;
                        self.#field_name = value;
                        Ok(())
                    }
                });
            }
        } else {
            return Err(syn::Error::new_spanned(field_ty, "unsupported field type"));
        }
    }

    Ok(matchs)
}