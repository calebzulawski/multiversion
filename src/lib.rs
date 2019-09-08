extern crate proc_macro;

mod dispatcher;
mod multiclones;
mod multiversion;

use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn multiversion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as multiversion::MultiVersion)
        .into_token_stream()
        .into()
}

#[proc_macro_attribute]
pub fn multiclones(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as multiclones::Config);
    let func = parse_macro_input!(input as ItemFn);
    multiclones::MultiClones::new(&config, &func)
        .into_token_stream()
        .into()
}
