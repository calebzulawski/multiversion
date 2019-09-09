extern crate proc_macro;

mod dispatcher;
mod multiversion;
mod target;
mod target_clones;

use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn multiversion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as multiversion::MultiVersion)
        .into_token_stream()
        .into()
}

#[proc_macro_attribute]
pub fn target_clones(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config = parse_macro_input!(attr as target_clones::Config);
    let func = parse_macro_input!(input as ItemFn);
    target_clones::TargetClones::new(config, &func)
        .into_token_stream()
        .into()
}
