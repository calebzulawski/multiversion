//! Implementation crate for `multiversion`.
extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod target;
mod util;

use quote::{quote, ToTokens};
use syn::{parse::Nothing, parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn multiversion(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    match multiversion::make_multiversioned_fn(attr.into(), func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn target(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let target = parse_macro_input!(attr as syn::LitStr);
    let func = parse_macro_input!(input as ItemFn);
    match target::make_target_fn(target, func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn inherit_target(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse_macro_input!(attr as Nothing);
    let func = parse_macro_input!(input as ItemFn);
    quote! {
        __multiversion::inherit_target! { #func }
    }
    .into()
}
