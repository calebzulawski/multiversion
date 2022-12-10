//! Implementation crate for `multiversion`.
extern crate proc_macro;

mod cfg;
mod dispatcher;
mod match_target;
mod multiversion;
mod target;
mod util;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Nothing, parse_macro_input, punctuated::Punctuated, ItemFn};

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

#[proc_macro]
pub fn selected_target(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Nothing);
    quote! {
        __multiversion::FEATURES
    }
    .into()
}

#[proc_macro_attribute]
pub fn target_cfg(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = TokenStream::from(attr);
    let input = TokenStream::from(input);
    quote! {
        __multiversion::target_cfg!{ [#attr] #input }
    }
    .into()
}

#[proc_macro_attribute]
pub fn target_cfg_attr(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = TokenStream::from(attr);
    let input = TokenStream::from(input);
    quote! {
        __multiversion::target_cfg_attr!{ [#attr] #input }
    }
    .into()
}

#[proc_macro]
pub fn target_cfg_f(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    quote! {
        __multiversion::target_cfg_f!{ #input }
    }
    .into()
}

#[proc_macro_attribute]
pub fn target_cfg_impl(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let meta = parse_macro_input!(attr with Punctuated::parse_terminated);
    let input = TokenStream::from(input);

    let meta = cfg::transform(meta);
    quote! {
        #[cfg(#meta)]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn target_cfg_attr_impl(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut meta = parse_macro_input!(attr with Punctuated::parse_terminated);
    let input = TokenStream::from(input);

    let attr = meta.pop().unwrap();
    let meta = cfg::transform(meta);
    quote! {
        #[cfg_attr(#meta, #attr)]
        #input
    }
    .into()
}

#[proc_macro]
pub fn target_cfg_f_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let meta = parse_macro_input!(input with Punctuated::parse_terminated);

    let meta = cfg::transform(meta);
    quote! {
        cfg!(#meta)
    }
    .into()
}

#[proc_macro]
pub fn match_target(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    quote! {
        __multiversion::match_target!{ #input }
    }
    .into()
}

#[proc_macro]
pub fn match_target_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let match_target = parse_macro_input!(input as match_target::MatchTarget);
    match_target.into_token_stream().into()
}
