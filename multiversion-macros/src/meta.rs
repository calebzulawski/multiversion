// Parses an attribute meta into Options
macro_rules! meta_parser {
    {
        $list:expr => [$($key:literal => $var:ident,)*]
    } => {
        $(let mut $var = None;)*
        for element in $list {
            match element {
                NestedMeta::Meta(meta) => match meta {
                    Meta::NameValue(nv) => {
                        let name = nv.path.get_ident()
                            .ok_or(Error::new(nv.path.span(), "unexpected key"))?
                            .to_string();
                        match name.as_str() {
                            $(
                                $key => {
                                    if $var.is_none() {
                                        $var = Some(&nv.lit);
                                    } else {
                                        return Err(Error::new(nv.path.span(), "key already provided"));
                                    }
                                }
                            )*
                            _ => return Err(Error::new(nv.path.span(), "unexpected key")),
                        };
                    }
                    _ => return Err(Error::new(meta.span(), "expected name-value pair")),
                }
                NestedMeta::Lit(lit) => return Err(Error::new(
                    lit.span(),
                    "unexpected literal, expected name-value pair",
                )),
            }
        }
    }
}
