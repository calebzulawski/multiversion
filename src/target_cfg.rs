use crate::target::Target;
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, Attribute, Block, Error, Lit, Meta,
    MetaNameValue, NestedMeta, Result,
};

pub(crate) fn process_target_cfg(target: Option<Target>, block: &mut Block) -> Result<()> {
    let mut visitor = ReplaceTargetCfg {
        target,
        result: Ok(()),
    };
    visitor.visit_block_mut(block);
    visitor.result
}

struct ReplaceTargetCfg {
    target: Option<Target>,
    result: Result<()>,
}

impl ReplaceTargetCfg {
    fn target_cfg_value(&self, nested: &NestedMeta) -> Result<bool> {
        match nested {
            NestedMeta::Meta(meta) => match meta {
                Meta::Path(path) => Err(Error::new(path.span(), "unexpected path")),
                Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                    if path.is_ident("target") {
                        if let Lit::Str(s) = lit {
                            let test_target = Some(Target::parse(s)?);
                            Ok(test_target == self.target)
                        } else {
                            Err(Error::new(lit.span(), "expected string literal"))
                        }
                    } else {
                        Err(Error::new(path.span(), "unknown key"))
                    }
                }
                Meta::List(list) => {
                    if list.path.is_ident("not") {
                        if list.nested.len() != 1 {
                            return Err(Error::new(
                                list.nested.span(),
                                "expected a single target_cfg predicate",
                            ));
                        }
                        self.target_cfg_value(list.nested.first().unwrap())
                            .map(|v| !v)
                    } else if list.path.is_ident("any") {
                        for v in list.nested.iter() {
                            if self.target_cfg_value(v)? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    } else if list.path.is_ident("all") {
                        for v in list.nested.iter() {
                            if !self.target_cfg_value(v)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    } else {
                        Err(Error::new(
                            list.path.span(),
                            "expected `not`, `any`, or `all`",
                        ))
                    }
                }
            },
            NestedMeta::Lit(lit) => Err(Error::new(lit.span(), "unexpected literal")),
        }
    }
}

impl VisitMut for ReplaceTargetCfg {
    fn visit_attribute_mut(&mut self, i: &mut Attribute) {
        if let Ok(Meta::List(list)) = i.parse_meta() {
            if list.path.is_ident("target_cfg") {
                self.result = self.result.clone().and_then(|_| {
                    if list.nested.len() != 1 {
                        return Err(Error::new(
                            list.nested.span(),
                            "expected a single target_cfg predicate",
                        ));
                    }
                    *i = if self.target_cfg_value(list.nested.first().unwrap())? {
                        parse_quote! { #[cfg(not(any()))] }
                    } else {
                        parse_quote! { #[cfg(any())] }
                    };
                    Ok(())
                })
            }
        }
    }
}
