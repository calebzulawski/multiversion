use crate::dispatcher::feature_fn_name;
use crate::target::Target;
use syn::{
    parse_quote, spanned::Spanned, Error, Ident, ItemFn, Lit, Meta, MetaList, NestedMeta, Path,
    Result, Stmt,
};

pub(crate) fn process_static_dispatch(item: &mut ItemFn, target: Option<&Target>) -> Result<()> {
    let mut retained = Vec::new();
    let mut bindings = Vec::<Stmt>::new();
    for attr in item.attrs.iter().cloned() {
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

        // parse the attribute
        match path.to_string().as_str() {
            "static_dispatch" => {
                meta_parser! {
                    &nested => [
                        "fn" => func,
                        "rename" => rename,
                    ]
                }
                let func = match func.ok_or(Error::new(nested.span(), "expected key 'fn'"))? {
                    Lit::Str(s) => s.parse_with(Path::parse_mod_style),
                    l => Err(Error::new(l.span(), "expected literal string")),
                }?;
                let rename: Option<Ident> = rename
                    .map(|lit| match lit {
                        Lit::Str(s) => s.parse(),
                        _ => Err(Error::new(lit.span(), "expected literal string")),
                    })
                    .transpose()?;

                // Build new source fn path
                let mut source = func.clone();

                // Get last ident in path
                let ident = &mut source.segments.last_mut().unwrap().ident;

                // Bound name is either the ident, or `rename` if it exists
                let binding = if let Some(rename) = rename {
                    rename.clone()
                } else {
                    ident.clone()
                };

                // Replace the last ident with the mangled name
                *ident = feature_fn_name(ident, target).1;
                bindings.push(parse_quote! { let #binding = #source; });
            }
            _ => {
                retained.push(attr);
            }
        }
    }

    // replace attributes
    item.attrs = retained;
    let block = &item.block;
    item.block = parse_quote! {
        {
            #(#bindings)*
            #block
        }
    };
    Ok(())
}
