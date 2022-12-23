use crate::target::Target;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    Error, Expr, ExprLit, Lit, LitStr, Pat, Result,
};

pub struct MatchTarget {
    features: LitStr,
    arms: Vec<(Target, Expr)>,
    default_target: Option<Expr>,
}

impl Parse for MatchTarget {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let mut arms = Vec::new();
        let mut default_target = None;

        while !input.is_empty() {
            let arm: syn::Arm = input.parse()?;
            if !arm.attrs.is_empty() {
                return Err(Error::new(arm.attrs[0].span(), "unexpected attribute"));
            }
            let pat = arm.pat;
            if let Some(guard) = arm.guard {
                return Err(Error::new(guard.0.span(), "unexpected guard"));
            }

            fn parse_target(e: &Expr) -> Result<Target> {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = e
                {
                    Target::parse(s)
                } else {
                    Err(Error::new(e.span(), "expected a string literal"))
                }
            }
            match pat {
                Pat::Lit(lit) => {
                    arms.push((parse_target(&lit.expr)?, *arm.body));
                }
                Pat::Or(or) => {
                    for case in or.cases.iter() {
                        if let Pat::Lit(lit) = case {
                            arms.push((parse_target(&lit.expr)?, *arm.body.clone()));
                        } else {
                            return Err(Error::new(case.span(), "expected a string literal"));
                        }
                    }
                }
                Pat::Wild(_) => {
                    default_target = Some(*arm.body);
                    if !input.is_empty() {
                        return Err(Error::new(input.span(), "unreachable targets"));
                    }
                }
                _ => return Err(Error::new(pat.span(), "expected string literal")),
            }
        }

        Ok(MatchTarget {
            features,
            arms,
            default_target,
        })
    }
}

impl ToTokens for MatchTarget {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut exprs = Vec::new();
        let mut not_targets = Vec::new();
        for (target, expr) in &self.arms {
            let arch = target.arch();
            let features = target.features();
            let selected_features = &self.features;
            let cfg = crate::cfg::transform(
                parse_quote! { #selected_features, all(target_arch = #arch #(, target_feature = #features)*) },
            );
            exprs.push(quote! {
                #[cfg(all(#cfg, not(any(#(#not_targets),*))))]
                { #expr }
            });
            not_targets.push(cfg);
        }
        let default_expr = if let Some(expr) = &self.default_target {
            quote! { #expr }
        } else {
            Error::new(Span::call_site(), "no matching target").to_compile_error()
        };

        quote! {
            {
                #(#exprs)*
                #[cfg(not(any(#(#not_targets),*)))]
                #default_expr
            }
        }
        .to_tokens(tokens)
    }
}
