//! Implementation crate for `multiversion`.
extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod safe_inner;
mod static_dispatch;
mod target;
mod target_cfg;
mod util;

use quote::ToTokens;
use syn::{parse_macro_input, ItemFn, ItemStruct};

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

#[proc_macro_derive(Dispatcher, attributes(target, crate_path))]
pub fn derive_dispatcher(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let dispatcher = parse_macro_input!(input as ItemStruct);
    match dispatcher::derive_dispatcher(dispatcher) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
