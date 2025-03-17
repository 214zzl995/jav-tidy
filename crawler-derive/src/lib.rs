use proc_macro::TokenStream;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed,
};

#[proc_macro_derive(Crawler, attributes(crawler))]
pub fn crawler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_derive(&input) {
        Ok(expanded) => expanded,
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_derive(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = &input.data
    else {
        return Err(syn::Error::new_spanned(
            input,
            "crawler can only be applied to structs",
        ));
    };

    let matchs = match build_try_set(named) {
        Ok(matchs) => matchs,
        Err(e) => return Ok(e.to_compile_error().into()),
    };

    let expanded = quote::quote! {
        use ::cinarium_crawler::CrawlerErr;

        impl ::cinarium_crawler::CrawlerData for #ident {
            fn try_set(&mut self, field: &str, values: Vec<String>) -> Result<(), CrawlerErr> {
                match field {
                    #(#matchs)*
                    _ => Ok(()),
                }
            }
        }
    };

    Ok(expanded.into())
}

fn build_try_set(named: &Punctuated<Field, Comma>) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut matchs = Vec::new();

    for field in named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_attrs = &field.attrs;

        if field_attrs.iter().any(|attr| {
            if attr.meta.path().is_ident("crawler") {
                let name = attr.parse_args::<syn::Ident>().unwrap();
                name == "skip"
            } else {
                false
            }
        }) {
            continue;
        };

        match field_ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let segment = &path.segments.last().unwrap();
                let ty = &segment.ident;

                if ty == "Vec" {
                    let ty = &segment.arguments;
                    if let syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments { args, .. },
                    ) = ty
                    {
                        let ty = args.first().unwrap();
                        if let syn::GenericArgument::Type(ty) = ty {
                            match ty {
                                syn::Type::Path(syn::TypePath { .. }) => {
                                    matchs.push(quote::quote! {
                                        stringify!(#field_name) => {
                                            self.#field_name = values.iter().map(|v| v.parse::<#ty>().unwrap()).collect();
                                            Ok(())
                                         }
                                    });
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        ty,
                                        "unsupported field type",
                                    ));
                                }
                            }
                        }
                    }
                } else if ty == "Option" {
                    matchs.push(quote::quote! {
                           stringify!(#field_name) => {
                                if values.len() > 1 {
                                   return Err(CrawlerErr::InvalidValueCount(stringify!(#field_name).to_string(), values.len()));
                                }
                                self.#field_name = values.get(0).map(|v| v.parse().unwrap());
                                Ok(())
                                }
                            });
                } else {
                    matchs.push(quote::quote! {
                        stringify!(#field_name) => {
                            if values.len() != 1 {
                                return Err(CrawlerErr::InvalidValueCount(stringify!(#field_name).to_string(), values.len()));
                            }
                            self.#field_name = values.get(0).unwrap().parse::<#ty>().unwrap();
                            Ok(())
                        }
                    });
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(field_ty, "unsupported field type"));
            }
        }
    }

    return Ok(matchs);
}
