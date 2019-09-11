extern crate proc_macro;

use crate::dispatcher::Dispatcher;
use crate::target::{parse_target_string, Target};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{token, Attribute, Block, FnArg, Ident, ItemFn, LitStr, Signature, Token, Visibility};

pub(crate) struct Config {
    targets: Vec<Target>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut targets = Vec::new();
        for target_string in Punctuated::<LitStr, Token![,]>::parse_terminated(&input)? {
            targets.extend(parse_target_string(&target_string)?)
        }
        Ok(Self { targets: targets })
    }
}

struct FunctionClone<'a> {
    target: Option<Target>,
    signature: Signature,
    real_function_ident: Ident,
    body: &'a Block,
}

impl ToTokens for FunctionClone<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_arch = self
            .target
            .as_ref()
            .map_or(TokenStream::new(), |x| x.target_arch());
        let target_features = self
            .target
            .as_ref()
            .map_or(TokenStream::new(), |x| x.target_features());
        let signature = &self.signature;
        let mut inner_signature = signature.clone();
        let mut recursion_helper_signature = signature.clone();
        let mut outer_signature = signature.clone();
        inner_signature.ident =
            Ident::new(&format!("{}_safe", self.signature.ident), Span::call_site());
        recursion_helper_signature.ident = self.real_function_ident.clone();
        outer_signature.unsafety = Some(token::Unsafe {
            span: Span::call_site(),
        });
        let inner_function_ident = &inner_signature.ident;
        let outer_function_ident = &outer_signature.ident;
        let body = &self.body;
        let argument_names = &outer_signature
            .inputs
            .iter()
            .map(|x| {
                if let FnArg::Typed(p) = x {
                    p.pat.as_ref()
                } else {
                    unimplemented!("member fn not supported")
                }
            })
            .collect::<Vec<_>>();
        // We create 3 different functions here: outer, inner, recursion_helper.
        // The outer function is the main one and in principle do not need
        // anything more than that one. The other two functions solve specific
        // problems.
        // The reason for the function inner is that we have to mark the outer
        // funtion as unsafe to be able to use target_feature, however to do not
        // not necessarily want the body to be unsafe. The solution is to place
        // it in an inner function, mark that function as inline(always) and
        // only ever call it from a single location.
        // The only remaining problem is now that of recursion. We would like
        // recursion to work as expected, without breaking the optimizations
        // above. We would also like it to call this function without going
        // through the dispatcher. The solution is to create a function in
        // scope with the same name and signature. This function will simply
        // call ourselves.
        tokens.extend(quote! {
            #target_arch
            #target_features
            #outer_signature {
                #target_arch
                #[inline(always)]
                #recursion_helper_signature {
                    unsafe {
                        #outer_function_ident(#(#argument_names),*)
                    }
                }

                #target_arch
                #[inline(always)]
                #inner_signature
                #body

                #inner_function_ident(#(#argument_names),*)
            }
        });
    }
}

pub(crate) struct TargetClones<'a> {
    attributes: &'a Vec<Attribute>,
    visibility: &'a Visibility,
    clones: Vec<FunctionClone<'a>>,
    dispatcher: Dispatcher,
}

impl ToTokens for TargetClones<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = (&self.attributes).iter();
        let visibility = &self.visibility;
        let signature = &self.dispatcher.signature;
        let clones = (&self.clones).iter();
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #(#attributes)*
            #visibility #signature {
                #(#clones)*
                #dispatcher
            }
        });
    }
}

impl<'a> TargetClones<'a> {
    pub fn new(config: Config, func: &'a ItemFn) -> Self {
        let mut clones = Vec::new();
        let mut functions = Vec::new();
        let real_function_ident = func.sig.ident.clone();
        let new_signature = move |target: Option<&Target>| {
            let mut signature = func.sig.clone();
            let mut name = signature.ident.to_string();
            if let Some(target) = target {
                name.push('_');
                name.push_str(target.arch_as_str());
                for feature in target.features_as_str() {
                    name.push('_');
                    name.push_str(&feature);
                }
            } else {
                name.push_str("_default");
            }
            signature.ident = Ident::new(&name, Span::call_site());
            signature
        };
        for target in config.targets {
            clones.push(FunctionClone {
                target: Some(target.clone()),
                signature: new_signature(Some(&target)),
                real_function_ident: real_function_ident.clone(),
                body: func.block.as_ref(),
            });
            functions.push((target, clones.last().unwrap().signature.ident.clone()));
        }
        // push default
        clones.push(FunctionClone {
            target: None,
            signature: new_signature(None),
            real_function_ident,
            body: func.block.as_ref(),
        });
        let default = clones.last().unwrap().signature.ident.clone();

        Self {
            attributes: &func.attrs,
            visibility: &func.vis,
            clones: clones,
            dispatcher: Dispatcher {
                signature: func.sig.clone(),
                functions: functions,
                default: default,
            },
        }
    }
}
