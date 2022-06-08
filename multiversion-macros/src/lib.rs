//! Implementation crate for `multiversion`.
extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod safe_inner;
mod target;
mod util;

use quote::ToTokens;
use syn::{parse_macro_input, punctuated::Punctuated, ItemFn, ItemMod};

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
    let target = parse_macro_input!(attr as Option<syn::Lit>);
    let func = parse_macro_input!(input as ItemFn);
    match target::make_target_fn(target, func) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn dispatcher(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let targets = parse_macro_input!(attr with Punctuated::parse_terminated);
    let dispatcher = parse_macro_input!(input as ItemMod);
    match dispatcher::derive_dispatcher(targets, dispatcher) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
