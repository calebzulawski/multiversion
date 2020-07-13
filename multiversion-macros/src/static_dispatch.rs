use crate::dispatcher::feature_fn_name;
use crate::target::Target;
use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Error, Expr, ItemFn, Result,
};

struct StaticDispatchVisitor<'a> {
    target: Option<&'a Target>,
    status: Result<()>,
}

impl<'a> StaticDispatchVisitor<'a> {
    pub fn new(target: Option<&'a Target>) -> Self {
        Self {
            target,
            status: Ok(()),
        }
    }

    pub fn status(self) -> Result<()> {
        self.status
    }
}

fn dispatch_impl(expr: &mut Expr, target: Option<&Target>) -> Result<()> {
    if let Expr::Macro(macro_expr) = expr {
        if let Some(path_ident) = macro_expr.mac.path.get_ident() {
            if path_ident.to_string().as_str() == "dispatch" {
                let mut call = macro_expr.mac.parse_body::<Expr>()?;
                let ident = match &mut call {
                    Expr::Call(ref mut call) => {
                        if let Expr::Path(ref mut function) = *call.func {
                            Ok(&mut function.path.segments.last_mut().unwrap().ident)
                        } else {
                            Err(Error::new(
                                call.func.span(),
                                "dispatching a function requires a direct function call",
                            ))
                        }
                    }
                    Expr::MethodCall(call) => Ok(&mut call.method),
                    Expr::Path(ref mut path) => {
                        Ok(&mut path.path.segments.last_mut().unwrap().ident)
                    }
                    _ => Err(Error::new(
                        call.span(),
                        "expected a function or method call",
                    )),
                }?;
                *ident = feature_fn_name(ident, target).1;
                *expr = call.into();
            }
        }
    }
    Ok(())
}

impl VisitMut for StaticDispatchVisitor<'_> {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if self.status.as_ref().ok().is_some() {
            if let Err(error) = dispatch_impl(i, self.target) {
                self.status = Err(error);
            }
            visit_mut::visit_expr_mut(self, i);
        }
    }
}

pub(crate) fn process_static_dispatch(item: &mut ItemFn, target: Option<&Target>) -> Result<()> {
    let mut visitor = StaticDispatchVisitor::new(target);
    visitor.visit_item_fn_mut(item);
    visitor.status()
}
