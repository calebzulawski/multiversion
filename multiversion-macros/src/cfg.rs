use quote::ToTokens;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, Lit, Meta, Result};

fn transform_recursive(features: &[&str], input: Meta) -> Result<Meta> {
    match input {
        Meta::NameValue(nv) => {
            if nv.path == parse_quote!(target_feature) {
                if let Expr::Lit(lit) = &nv.value {
                    if let Lit::Str(lit) = &lit.lit {
                        if features.contains(&lit.value().as_str()) {
                            return Ok(parse_quote! { all() });
                        }
                    }
                }
            }
            Ok(Meta::NameValue(nv))
        }
        Meta::List(mut list) => {
            let mut metas = list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)?;
            for meta in metas.iter_mut() {
                *meta = transform_recursive(features, meta.clone())?;
            }
            list.tokens = metas.into_token_stream();
            Ok(Meta::List(list))
        }
        input => Ok(input),
    }
}

pub(crate) fn transform(mut input: Punctuated<Meta, Comma>) -> Result<Meta> {
    assert_eq!(input.len(), 2);

    let features = if let Expr::Lit(features) = &input[0].require_name_value()?.value {
        if let Lit::Str(features) = &features.lit {
            Some(features.value())
        } else {
            None
        }
    } else {
        None
    };

    let features = features.expect("couldn't parse first argument");
    let features = features.split(',').collect::<Vec<&str>>();

    transform_recursive(&features, input.pop().unwrap().into_value())
}
