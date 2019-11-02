use syn::{parse_quote, Error, FnArg, Pat, Result, Signature, TypeBareFn};

pub(crate) fn args_from_signature<'a>(sig: &'a Signature) -> Result<Vec<&'a Pat>> {
    sig.inputs
        .iter()
        .map(|x| match x {
            FnArg::Receiver(rec) => Err(Error::new(rec.self_token.span, "member fn not supported")),
            FnArg::Typed(arg) => Ok(arg.pat.as_ref()),
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn fn_type_from_signature(sig: &Signature) -> TypeBareFn {
    let lifetimes = sig.generics.lifetimes().collect::<Vec<_>>();
    let args = sig.inputs.iter();
    TypeBareFn {
        lifetimes: if lifetimes.is_empty() {
            None
        } else {
            Some(parse_quote! { <#(#lifetimes),*> })
        },
        unsafety: sig.unsafety,
        abi: sig.abi.clone(),
        fn_token: sig.fn_token,
        paren_token: sig.paren_token,
        inputs: parse_quote! { #(#args),* },
        variadic: sig.variadic.clone(),
        output: sig.output.clone(),
    }
}
