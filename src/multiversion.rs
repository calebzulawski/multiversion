use crate::dispatcher::Dispatcher;
use crate::target::Target;
use crate::util;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::Parse, parse::ParseStream, parse_quote, punctuated::Punctuated, token, ItemFn, LitStr,
    Path, Result,
};

pub(crate) struct Specialization {
    target: Target,
    _fat_arrow_token: token::FatArrow,
    path: Path,
}

impl Parse for Specialization {
    fn parse(input: ParseStream) -> Result<Self> {
        let target_str = input.parse::<LitStr>()?;
        Ok(Self {
            target: Target::parse(&target_str)?,
            _fat_arrow_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

pub(crate) struct Config {
    specializations: Punctuated<Specialization, token::Comma>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            specializations: input.parse_terminated(Specialization::parse)?,
        })
    }
}

pub(crate) fn make_multiversioned_fn(config: Config, func: ItemFn) -> Result<TokenStream> {
    let normalized_sig = util::normalize_signature(&func.sig)?;
    let args = util::args_from_signature(&normalized_sig)?;
    let fn_params = util::fn_params(&func.sig);
    Ok(Dispatcher {
        attrs: func.attrs,
        vis: func.vis,
        sig: func.sig.clone(),
        specializations: config
            .specializations
            .iter()
            .map(
                |Specialization { target, path, .. }| crate::dispatcher::Specialization {
                    target: target.clone(),
                    block: parse_quote! {
                        {
                            unsafe { #path::<#(#fn_params),*>(#(#args),*) }
                        }
                    },
                    normalize: true,
                },
            )
            .collect(),
        default: *func.block,
    }
    .to_token_stream())
}
