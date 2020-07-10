use crate::target::Target;
use syn::{
    parse_quote,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Error, Expr, ExprBlock, ExprCall, ItemFn, Path, Result,
};

struct StaticDispatchVisitor<'a> {
    crate_path: &'a Path,
    target: Option<&'a Target>,
    status: Result<()>,
}

impl<'a> StaticDispatchVisitor<'a> {
    pub fn new(crate_path: &'a Path, target: Option<&'a Target>) -> Self {
        Self {
            crate_path,
            target,
            status: Ok(()),
        }
    }

    pub fn status(self) -> Result<()> {
        self.status
    }
}

impl VisitMut for StaticDispatchVisitor<'_> {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if self.status.as_ref().ok().is_some() {
            if let Expr::Macro(expr) = i {
                if let Some(path_ident) = expr.mac.path.get_ident() {
                    let crate_path = &self.crate_path;
                    let features = if let Some(target) = self.target {
                        target.list_features()
                    } else {
                        &[]
                    };
                    match path_ident.to_string().as_str() {
                        "token" => {
                            if expr.mac.tokens.is_empty() {
                                *i = parse_quote! { unsafe { #crate_path::CpuFeatures::new(&[#(#features),*]) } };
                            } else {
                                self.status = Err(Error::new(
                                    expr.span(),
                                    "`tokens!()` helper macro doesn't take any arguments",
                                ));
                            }
                        }
                        "dispatch" => match expr.mac.parse_body::<ExprCall>() {
                            Ok(call) => {
                                let block: ExprBlock = parse_quote! {
                                    {
                                        const __TOKEN: #crate_path::CpuFeatures = unsafe { #crate_path::CpuFeatures::new(&[#(#features),*]) };
                                        #crate_path::dispatch!(__TOKEN => #call)
                                    }
                                };
                                *i = block.into();
                            }
                            Err(error) => {
                                self.status = Err(error);
                            }
                        },
                        _ => { /* skip this macro */ }
                    }
                }
            }
        }
        visit_mut::visit_expr_mut(self, i);
    }
}

pub(crate) fn process_static_dispatch(
    item: &mut ItemFn,
    crate_path: &Path,
    target: Option<&Target>,
) -> Result<()> {
    let mut visitor = StaticDispatchVisitor::new(crate_path, target);
    visitor.visit_item_fn_mut(item);
    visitor.status()
}
