extern crate proc_macro;

use crate::{dispatcher::Dispatcher, dispatcher::Specialization, target::Target};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, ItemFn, LitStr, Result, Token,
};

pub(crate) struct Config {
    targets: Vec<Target>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut targets = Vec::new();
        for target_string in Punctuated::<LitStr, Token![,]>::parse_terminated(&input)? {
            targets.push(Target::parse(&target_string)?)
        }
        Ok(Self { targets })
    }
}

pub(crate) fn make_target_clones_fn(config: Config, func: ItemFn) -> TokenStream {
    let specializations = config
        .targets
        .iter()
        .map(|target| Specialization {
            target: target.clone(),
            block: func.block.as_ref().clone(),
        })
        .collect();
    let dispatcher = Dispatcher {
        attrs: func.attrs,
        vis: func.vis,
        sig: func.sig,
        specializations,
        default: *func.block,
    };
    quote! { #dispatcher }
}
