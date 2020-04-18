use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, BareFnArg, Error, FnArg, GenericParam,
    Ident, Lifetime, Pat, PatIdent, PatType, Result, ReturnType, Signature, Type, TypeBareFn,
};

pub(crate) fn normalize_signature(sig: &Signature) -> Signature {
    let args = sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, x)| match x {
            FnArg::Receiver(_) => x.clone(),
            FnArg::Typed(arg) => FnArg::Typed(PatType {
                pat: Box::new(Pat::Ident(PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident: match arg.pat.as_ref() {
                        Pat::Ident(pat) => pat.ident.clone(),
                        _ => Ident::new(&format!("__multiversion_arg_{}", i), x.span()),
                    },
                    subpat: None,
                })),
                ..arg.clone()
            }),
        })
        .collect::<Vec<_>>();
    Signature {
        inputs: parse_quote! { #(#args),* },
        ..sig.clone()
    }
}

pub(crate) fn args_from_signature<'a>(sig: &'a Signature) -> Result<Vec<&'a Pat>> {
    sig.inputs
        .iter()
        .map(|x| match x {
            FnArg::Receiver(rec) => Err(Error::new(rec.self_token.span, "member fn not supported")),
            FnArg::Typed(arg) => Ok(arg.pat.as_ref()),
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn impl_trait_present(sig: &Signature) -> bool {
    if let ReturnType::Type(_, ty) = &sig.output {
        if let Type::ImplTrait(_) = **ty {
            return true;
        }
    }
    sig.inputs.iter().any(|arg| {
        if let FnArg::Typed(pat) = arg {
            if let Type::ImplTrait(_) = *pat.ty {
                return true;
            }
        }
        false
    })
}

struct LifetimeRenamer;

impl VisitMut for LifetimeRenamer {
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        i.ident = Ident::new(&format!("__mv_inner_{}", i.ident), i.ident.span());
    }
}

pub(crate) fn fn_type_from_signature(sig: &Signature) -> Result<TypeBareFn> {
    let lifetimes = sig.generics.lifetimes().collect::<Vec<_>>();
    let args = sig
        .inputs
        .iter()
        .map(|x| {
            Ok(BareFnArg {
                attrs: Vec::new(),
                name: None,
                ty: match x {
                    FnArg::Receiver(rec) => {
                        Err(Error::new(rec.self_token.span, "member fn not supported"))
                    }
                    FnArg::Typed(arg) => Ok(arg.ty.as_ref().clone()),
                }?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut fn_ty = TypeBareFn {
        lifetimes: if lifetimes.is_empty() {
            None
        } else {
            Some(parse_quote! { for<#(#lifetimes),*> })
        },
        unsafety: sig.unsafety,
        abi: sig.abi.clone(),
        fn_token: sig.fn_token,
        paren_token: sig.paren_token,
        inputs: parse_quote! { #(#args),* },
        variadic: sig.variadic.clone(),
        output: sig.output.clone(),
    };
    LifetimeRenamer {}.visit_type_bare_fn_mut(&mut fn_ty);
    Ok(fn_ty)
}

pub(crate) fn fn_params(sig: &Signature) -> Vec<Ident> {
    sig.generics
        .params
        .iter()
        .filter_map(|x| match x {
            GenericParam::Type(ty) => Some(ty.ident.clone()),
            GenericParam::Const(c) => Some(c.ident.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn await_tokens() -> TokenStream {
    let kw = Ident::new("await", Span::call_site());
    quote! { .#kw }
}
