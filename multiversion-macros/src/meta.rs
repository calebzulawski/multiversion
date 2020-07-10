use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Error, Ident, Lit, Meta,
    MetaList, NestedMeta, Path, Result,
};

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

pub(crate) fn parse_attributes<I, F>(attrs: I, mut f: F) -> Result<Vec<Attribute>>
where
    I: Iterator<Item = Attribute>,
    F: FnMut(&Ident, &Punctuated<NestedMeta, Comma>) -> Result<bool>,
{
    let mut retained = Vec::new();
    for attr in attrs {
        // if not in meta list form, ignore the attribute
        let MetaList { path, nested, .. } = if let Ok(Meta::List(list)) = attr.parse_meta() {
            list
        } else {
            retained.push(attr);
            continue;
        };

        // if meta path isn't just an ident, ignore the attribute
        let path = if let Some(ident) = path.get_ident() {
            ident
        } else {
            retained.push(attr);
            continue;
        };

        if !f(path, &nested)? {
            retained.push(attr);
        }
    }
    Ok(retained)
}

pub(crate) fn parse_crate_path(nested: &Punctuated<NestedMeta, Comma>) -> Result<Path> {
    meta_parser! {
        nested => [
            "path" => crate_path,
        ]
    }
    if let Lit::Str(crate_path) =
        crate_path.ok_or_else(|| Error::new(nested.span(), "expected key 'path'"))?
    {
        Ok(crate_path.parse()?)
    } else {
        Err(Error::new(crate_path.span(), "expected literal string"))
    }
}
