use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, Expr, FnArg, GenericParam, Ident, Item,
    ItemFn, Lifetime, Pat, PatIdent, PatType, Signature,
};

struct HasSelfType(bool);

impl VisitMut for HasSelfType {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        self.0 |= ident == "Self"
    }

    fn visit_item_mut(&mut self, _: &mut Item) {
        // Nested items may have `Self` tokens
    }
}

pub(crate) fn is_associated_fn(item: &mut ItemFn) -> bool {
    if item.sig.receiver().is_some() {
        return true;
    }
    let mut v = HasSelfType(false);
    v.visit_item_fn_mut(item);
    v.0
}

pub(crate) fn arg_exprs(sig: &Signature) -> Vec<Expr> {
    sig.inputs
        .iter()
        .map(|x| match x {
            FnArg::Receiver(rec) => {
                let self_token = rec.self_token;
                parse_quote! { #self_token }
            }
            FnArg::Typed(arg) => {
                if let Pat::Ident(ident) = &*arg.pat {
                    let ident = &ident.ident;
                    parse_quote! { #ident }
                } else {
                    panic!("pattern should have been ident")
                }
            }
        })
        .collect()
}

pub(crate) fn normalize_signature(sig: &Signature) -> (Signature, Vec<Expr>) {
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
    let sig = Signature {
        inputs: parse_quote! { #(#args),* },
        ..sig.clone()
    };
    let callable_args = arg_exprs(&sig);
    (sig, callable_args)
}

struct LifetimeRenamer;

impl VisitMut for LifetimeRenamer {
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        i.ident = Ident::new(&format!("__mv_inner_{}", i.ident), i.ident.span());
    }
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
