use crate::{dispatcher::feature_fn_name, target::Target};
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, Attribute, Block, Error, Ident, ItemUse,
    Lit, Meta, MetaNameValue, NestedMeta, Result, UseName, UsePath, UseRename, UseTree,
};

pub(crate) fn process_helper_attributes(target: Option<Target>, block: &mut Block) -> Result<()> {
    let mut visitor = HelperAttributeVisitor {
        target,
        result: Ok(()),
    };
    visitor.visit_block_mut(block);
    visitor.result
}

struct HelperAttributeVisitor {
    target: Option<Target>,
    result: Result<()>,
}

impl HelperAttributeVisitor {
    fn rebuild_use_tree(&self, tree: &UseTree) -> Result<ItemUse> {
        fn finish(
            idents: Vec<&Ident>,
            name: &Ident,
            rename: &Ident,
            target: Option<&Target>,
        ) -> ItemUse {
            let fn_name = feature_fn_name(&name, target);
            if idents.is_empty() {
                parse_quote! { use #fn_name as #rename; }
            } else {
                parse_quote! { use #(#idents)::*::#fn_name as #rename; }
            }
        }
        fn detail<'a>(
            tree: &'a UseTree,
            mut idents: Vec<&'a Ident>,
            target: Option<&Target>,
        ) -> Result<ItemUse> {
            match tree {
                UseTree::Path(UsePath { ident, tree, .. }) => {
                    idents.push(ident);
                    detail(tree, idents, target)
                }
                UseTree::Name(UseName { ref ident }) => Ok(finish(idents, ident, ident, target)),
                UseTree::Rename(UseRename { ident, rename, .. }) => {
                    Ok(finish(idents, ident, rename, target))
                }
                _ => Err(Error::new(
                    tree.span(),
                    "unsupported use statement for #[static_dispatch]",
                )),
            }
        }
        detail(tree, Vec::new(), self.target.as_ref())
    }

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
                            Err(Error::new(
                                list.nested.span(),
                                "expected a single target_cfg predicate",
                            ))?
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

impl VisitMut for HelperAttributeVisitor {
    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        let before = i.attrs.len();
        i.attrs
            .retain(|attr| *attr != parse_quote! { #[static_dispatch] });
        if i.attrs.len() < before {
            self.result = self.result.clone().and_then(|_| {
                *i = self.rebuild_use_tree(&i.tree)?;
                Ok(())
            });
        }
    }

    fn visit_attribute_mut(&mut self, i: &mut Attribute) {
        if let Ok(Meta::List(list)) = i.parse_meta() {
            if list.path.is_ident("target_cfg") {
                self.result = self.result.clone().and_then(|_| {
                    if list.nested.len() != 1 {
                        Err(Error::new(
                            list.nested.span(),
                            "expected a single target_cfg predicate",
                        ))?
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
