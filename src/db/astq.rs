use crate::parse;
use crate::ast;
use crate::common::*;

use std::sync::Arc;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: salsa::Database {
    #[salsa::input]
    fn source(&self) -> Arc<String>;

    fn package_ast(&self) -> VirdantResult<ast::Package>;
    fn moddef_ast(&self, moddef: Ident) -> VirdantResult<ast::ModDef>;
    fn moddef_component_ast(&self, moddef: Ident, component: Ident) -> VirdantResult<ast::SimpleComponent>;

    fn moddef_components(&self, moddef: Ident) -> VirdantResult<Vec<ast::SimpleComponent>>;
    fn moddef_submodules(&self, moddef: Ident) -> VirdantResult<Vec<ast::Submodule>>;

    fn moddef_wire(&self, moddef: Ident, target: Path) -> VirdantResult<ast::Wire>;
    fn moddef_wire_exprinst(&self, moddef: Ident, target: Path, subexpr_path: ExprPath) -> VirdantResult<Arc<ast::Expr>>;

    fn moddef_wire_expr(&self, moddef: Ident, target: Path) -> VirdantResult<Arc<ast::Expr>>;
}

fn package_ast(db: &dyn AstQ) -> Result<ast::Package, VirdantError> {
    let input = db.source();
    parse::parse_package(&input)
}

fn moddef_ast(db: &dyn AstQ, moddef: Ident) -> Result<ast::ModDef, VirdantError> {
    let package = db.package_ast()?;
    let mut result: Option<ast::ModDef> = None;

    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == moddef {
                    if result.is_none() {
                        result = Some(moddef_ast.clone());
                    } else {
                        return Err(VirdantError::Other("Uh oh".into()));
                    }
                }
            },
            ast::Item::StructTypeDef(_structtypedef) => (),
        }
    }

    if let Some(moddef) = result {
        Ok(moddef)
    } else {
        Err(VirdantError::Other(format!("Unknown moddef {moddef}")))
    }
}

fn moddef_component_ast(db: &dyn AstQ, moddef: Ident, component: Ident) -> Result<ast::SimpleComponent, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(c) if c.name == component => return Ok(c.clone()),
            _ => (),
        }
    }
    Err(VirdantError::Other(format!("No such moddef {}", moddef)))
}


fn moddef_components(db: &dyn AstQ, moddef: Ident) -> VirdantResult<Vec<ast::SimpleComponent>> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut results = vec![];
    for decl in &moddef_ast.decls {
        if let ast::Decl::SimpleComponent(component) = decl {
            results.push(component.clone());
        }
    }
    Ok(results)
}

fn moddef_submodules(db: &dyn AstQ, moddef: Ident) -> VirdantResult<Vec<ast::Submodule>> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut results = vec![];
    for decl in &moddef_ast.decls {
        if let ast::Decl::Submodule(submodule) = decl {
            results.push(submodule.clone());
        }
    }
    Ok(results)
}

fn moddef_wire(db: &dyn AstQ, moddef: Ident, target: Path) -> VirdantResult<ast::Wire> {
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    let mut wire_asts = vec![];

    for decl in &moddef_ast.decls {
        if let ast::Decl::Wire(wire @ ast::Wire(wire_target, _wire_type, _expr)) = decl {
            if wire_target == &target {
                wire_asts.push(wire.clone());
            }
        }
    }

    if wire_asts.len() < 1 {
        Err(VirdantError::Other(format!("No wire for: {target} in {moddef}")))
    } else if wire_asts.len() > 1 {
        Err(VirdantError::Other(format!("Multiple wires for: {target} in {moddef}")))
    } else {
        Ok(wire_asts[0].clone())
    }
}

fn moddef_wire_exprinst(
    db: &dyn AstQ,
    moddef: Ident,
    target: Path,
    expr_path: ExprPath,
) -> VirdantResult<Arc<ast::Expr>> {
    let ast::Wire(_target, _wire_type, expr) = db.moddef_wire(moddef, target)?;
    let expr = get_subexpr(expr, &expr_path);
    Ok(expr)
}

fn moddef_wire_expr(db: &dyn AstQ, moddef: Ident, target: Path) -> VirdantResult<Arc<ast::Expr>> {
    let ast::Wire(_target, _wire_type, expr) = db.moddef_wire(moddef, target)?;
    Ok(expr)
}

pub fn get_subexpr(expr: Arc<ast::Expr>, path: &ExprPath) -> Arc<ast::Expr> {
    let mut result = expr.clone();

    for idx in path.walk() {
        result = result.subexpr(*idx);
    }

    result
}

impl ast::Expr {
    fn subexpr(&self, i: usize) -> Arc<ast::Expr> {
        match self {
            ast::Expr::Reference(_path) => unreachable!(),
            ast::Expr::Word(_lit) => unreachable!(),
            ast::Expr::Vec(_) => todo!(),
            ast::Expr::Struct(_, _) => todo!(),
            ast::Expr::MethodCall(subject, _method, args) => {
                if i == 0 {
                    subject.clone()
                } else {
                    args[i-1].clone()
                }
            },
            ast::Expr::As(e, _typ) => {
                assert_eq!(i, 0);
                e.clone()
            },
            ast::Expr::Idx(e, _i) => {
                assert_eq!(i, 0);
                e.clone()
            }
            ast::Expr::IdxRange(e, _j, _i) => {
                assert_eq!(i, 0);
                e.clone()
            }
            ast::Expr::Cat(es) => {
                es[i].clone()
            },
            ast::Expr::If(c, a, b) => {
                if i == 0 {
                    c.clone()
                } else if i == 1 {
                    a.clone()
                } else if i == 1 {
                    b.clone()
                } else {
                    unreachable!()
                }
            }
        }
    }
}
