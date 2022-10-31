use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, visit_mut::VisitMut, Lit, Meta, NestedMeta,
};

fn parse_features(features: &NestedMeta) -> Vec<String> {
    let lit = match features {
        NestedMeta::Lit(lit) => lit,
        _ => unimplemented!(),
    };
    let s = match lit {
        Lit::Str(s) => s,
        _ => unimplemented!(),
    };
    s.value().split(',').map(str::to_string).collect()
}

pub(crate) fn transform(input: Punctuated<NestedMeta, Comma>) -> NestedMeta {
    assert!(input.len() == 2);

    let features = parse_features(&input[0]);
    let mut attr = input.into_iter().nth(1).unwrap();

    struct Visitor(Vec<String>);

    impl VisitMut for Visitor {
        fn visit_meta_mut(&mut self, i: &mut Meta) {
            // replace instances of `target_feature = "..."` when they match a detected target
            // feature.
            if let Meta::NameValue(nv) = i {
                if nv.path == parse_quote!(target_feature) {
                    if let Lit::Str(s) = &nv.lit {
                        if self.0.contains(&s.value()) {
                            *i = parse_quote! { all() };
                        }
                    }
                }
            }

            syn::visit_mut::visit_meta_mut(self, i);
        }
    }

    Visitor(features).visit_nested_meta_mut(&mut attr);
    attr
}
