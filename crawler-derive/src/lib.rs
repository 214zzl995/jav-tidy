extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Crawler, attributes(crawler))]
pub fn crawler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_impl::expand_derive(&input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

mod derive_impl;