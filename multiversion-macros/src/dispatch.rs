use crate::dispatcher::{dispatch_fn_name, dispatch_index_fn_name};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    token::FatArrow,
    Error, Expr, Result,
};

pub(crate) struct Config {
    token: Expr,
    _arrow: FatArrow,
    function: Expr,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            token: input.parse()?,
            _arrow: input.parse()?,
            function: input.parse()?,
        })
    }
}

pub(crate) fn dispatch(config: Config) -> Result<Expr> {
    let token = config.token;
    match config.function {
        Expr::Call(call) => {
            let (index_fn, dispatch_fn) = if let Expr::Path(function) = *call.func {
                let ident = &function.path.segments.last().unwrap().ident;
                let mut index_fn = function.clone();
                index_fn.path.segments.last_mut().unwrap().ident = dispatch_index_fn_name(&ident);
                let mut dispatch_fn = function.clone();
                dispatch_fn.path.segments.last_mut().unwrap().ident = dispatch_fn_name(&ident);
                Ok((index_fn, dispatch_fn))
            } else {
                Err(Error::new(
                    call.func.span(),
                    "dispatching a function requires a direct function call",
                ))
            }?;
            let args = call.args;
            Ok(parse_quote! {
                {
                    const __MULTIVERSION_FN_INDEX: usize = #index_fn(#token);
                    unsafe { #dispatch_fn(__MULTIVERSION_FN_INDEX, #args) }
                }
            })
        }
        other => Err(Error::new(
            other.span(),
            "expected a function call expression",
        )),
    }
}
