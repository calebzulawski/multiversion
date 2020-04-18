use quote::quote;
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, Error, Ident, Item, ItemFn, Result,
    Signature, Visibility,
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

pub fn process_safe_inner(mut item: ItemFn) -> Result<Vec<ItemFn>> {
    let safe_inner_span = {
        if let Some((idx, attr)) = item
            .attrs
            .iter()
            .enumerate()
            .find(|(_, attr)| **attr == parse_quote! { #[safe_inner] })
        {
            if item.sig.unsafety.is_none() {
                Err(Error::new(
                    attr.span(),
                    "#[safe_inner] may only be used on unsafe fn",
                ))
            } else {
                let span = attr.span();
                item.attrs.remove(idx);
                Ok(Some(span))
            }
        } else {
            Ok(None)
        }
    }?;
    let associated = is_associated_fn(&mut item);
    if let Some(safe_inner_span) = safe_inner_span {
        // create safe function
        // copy #[cfg] attributes
        let attrs = item
            .attrs
            .iter()
            .filter_map(|attr| {
                if let Ok(meta) = attr.parse_meta() {
                    if *meta.path() == parse_quote! { cfg } {
                        Some(attr.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .chain(std::iter::once(parse_quote! { #[inline(always)] }))
            .collect();
        let safe_fn = ItemFn {
            attrs,
            vis: Visibility::Inherited,
            sig: Signature {
                unsafety: None,
                ident: Ident::new(&format!("__safe_inner_{}", item.sig.ident), safe_inner_span),
                ..item.sig.clone()
            },
            block: item.block,
        };

        // create unsafe function
        let (unsafe_sig, args) = crate::util::normalize_signature(&item.sig);
        let maybe_await = item.sig.asyncness.map(|_| crate::util::await_tokens());
        let maybe_self = if associated {
            quote! { Self:: }
        } else {
            Default::default()
        };
        let safe_ident = &safe_fn.sig.ident;
        let fn_params = crate::util::fn_params(&unsafe_sig);
        let unsafe_fn = ItemFn {
            block: parse_quote! {
                {
                    #maybe_self#safe_ident::<#(#fn_params),*>(#(#args),*)#maybe_await
                }
            },
            sig: unsafe_sig,
            ..item
        };
        Ok(vec![unsafe_fn, safe_fn])
    } else {
        Ok(vec![item])
    }
}
