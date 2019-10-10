extern crate proc_macro;

use crate::dispatcher::Dispatcher;
use crate::target::{parse_target_string, Target};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{
    token, Attribute, Block, Expr, FnArg, Ident, ItemFn, LitStr, Signature, Token, Visibility,
};

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
    body: &'a Block,
}

impl FunctionClone<'_> {
    pub fn name(&self) -> Ident {
        Ident::new(
            &if self.target.is_some() && self.target.as_ref().unwrap().has_features_specified() {
                format!(
                    "feature_{}",
                    self.target.as_ref().unwrap().features_string()
                )
            } else {
                "default".to_string()
            },
            Span::call_site(),
        )
    }

    pub fn unsafety(&self) -> Option<token::Unsafe> {
        if self.target.is_some() && self.target.as_ref().unwrap().has_features_specified() {
            Some(token::Unsafe {
                span: Span::call_site(),
            })
        } else {
            self.signature.unsafety
        }
    }
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

        // Outer fn has a descriptive name and is unsafe if features are specified
        let outer_signature = Signature {
            ident: self.name(),
            unsafety: self.unsafety(),
            ..signature.clone()
        };
        let outer_function_ident = &outer_signature.ident;

        // Implementation fn signature matches the original fn signature, but with a different name
        let impl_signature = Signature {
            ident: Ident::new("__fn_impl", Span::call_site()),
            ..signature.clone()
        };
        let impl_function_ident = &impl_signature.ident;
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

        // Recursion helper fn signature matches the original function signature
        let recursion_helper_signature = signature.clone();

        // We create the outer function here, containing two inner functions, the implementation
        // and the recursion helper.
        //
        // The outer function is invoked by the dispatcher. It must be marked unsafe since it uses
        // the `target_feature` attribute.
        //
        // The implementation function contains the actual implementation of the function. This
        // function has the same safety as the original function, maintaining normal safety
        // guarantees inside the unsafe fn required by the `target_feature` attribute.
        //
        // The recursion helper function has the same name as the original function, allowing
        // recursive functions to skip runtime dispatch.  This function must be separate from the
        // implementation function, since recursion must call the outer function marked with the
        // `target_feature` attribute. See Rust #53117 for more info.
        tokens.extend(quote! {
            #target_arch
            #target_features
            pub #outer_signature {
                #[inline(always)]
                #recursion_helper_signature {
                    unsafe {
                        #outer_function_ident(#(#argument_names),*)
                    }
                }

                #[inline(always)]
                #impl_signature
                #body

                #impl_function_ident(#(#argument_names),*)
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
        let name = &signature.ident;
        let clones = (&self.clones).iter();
        let dispatcher = &self.dispatcher;
        tokens.extend(quote! {
            #visibility mod #name {
                use super::*;
                #(#clones)*
            }
            #(#attributes)*
            #visibility #signature {
                #dispatcher
            }
        });
    }
}

impl<'a> TargetClones<'a> {
    pub fn new(config: Config, func: &'a ItemFn) -> Self {
        let module = func.sig.ident.clone();
        let prepend_module = move |name| syn::parse2::<Expr>(quote! { #module::#name }).unwrap();
        let mut clones = Vec::new();
        let mut functions = Vec::new();
        for target in config.targets {
            clones.push(FunctionClone {
                target: Some(target.clone()),
                signature: func.sig.clone(),
                body: func.block.as_ref(),
            });
            functions.push((target, prepend_module(clones.last().unwrap().name())));
        }
        // push default
        clones.push(FunctionClone {
            target: None,
            signature: func.sig.clone(),
            body: func.block.as_ref(),
        });
        let default = prepend_module(clones.last().unwrap().name());

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
