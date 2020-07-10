//! Implementation crate for `multiversion`.
extern crate proc_macro;

#[macro_use]
mod meta;

mod dispatch;
mod dispatcher;
mod multiversion;
mod safe_inner;
mod static_dispatch;
mod target;
mod target_cfg;
mod util;

use quote::ToTokens;
use syn::{parse::Nothing, parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn multiversion(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse_macro_input!(attr as Nothing);
    let func = parse_macro_input!(input as ItemFn);
    match multiversion::make_multiversioned_fn(func) {
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

#[proc_macro]
pub fn dispatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let config = parse_macro_input!(input as dispatch::Config);
    match dispatch::dispatch(config) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
