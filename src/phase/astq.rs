use crate::ast::ComponentKind;
use crate::parse;
use crate::ast;
use crate::ast::Ast;
use crate::common::*;
use crate::virdant_error;
use super::*;

#[salsa::query_group(AstQStorage)]
pub trait AstQ: sourceq::SourceQ {
    fn packages(&self) -> Vec<PackageId>;

    fn package_ast(&self, package_id: PackageId) -> VirdantResult<Ast<ast::Package>>;
    fn item_ast(&self, item_id: ItemId) -> VirdantResult<ast::Item>;
    fn moddef_ast(&self, moddef_id: ModDefId) -> VirdantResult<Ast<ast::ModDef>>;
    fn uniondef_ast(&self, uniondef_id: UnionDefId) -> VirdantResult<Ast<ast::UnionDef>>;
    fn structdef_ast(&self, structdef_id: StructDefId) -> VirdantResult<Ast<ast::StructDef>>;
    fn portdef_ast(&self, portdef_id: PortDefId) -> VirdantResult<Ast<ast::PortDef>>;

    fn component_ast(&self, component_id: ComponentId) -> VirdantResult<Ast<ast::Component>>;

    fn wire_ast(&self, moddef_id: ModDefId, path_id: Path) -> VirdantResult<Option<Ast<ast::Wire>>>;
}

fn packages(db: &dyn AstQ) -> Vec<PackageId> {
    let mut packages: Vec<String> = vec![];

    for package_name in db.sources().keys() {
        packages.push(package_name.clone());
    }

    packages.sort();
    packages.into_iter().map(|package| PackageId::from_ident(package.into())).collect()
}

fn package_ast(db: &dyn AstQ, package_id: PackageId) -> VirdantResult<Ast<ast::Package>> {
    eprintln!("package_ast({package_id})");
    let sources = db.sources();
    let package_name = package_id.name().to_string();
    if let Some(input) = sources.get(&package_name) {
        parse::parse_package(&package_name, &input)
    } else {
        Err(virdant_error!("TODO package_ast"))
    }
}

fn item_ast(db: &dyn AstQ, item_id: ItemId) -> VirdantResult<ast::Item> {
    let package_ast = db.package_ast(item_id.package())?;
    let mut result: Option<ast::Item> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == item_id.name() {
                    if result.is_none() {
                        result = Some(ast::Item::ModDef(moddef_ast.clone()));
                    } else {
                        return Err(virdant_error!("Duplicate item found: {item_id}"));
                    }
                }
            },
            ast::Item::UnionDef(uniondef_ast) => {
                if uniondef_ast.name == item_id.name() {
                    if result.is_none() {
                        result = Some(ast::Item::UnionDef(uniondef_ast.clone()));
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            ast::Item::StructDef(structdef_ast) => {
                if structdef_ast.name == item_id.name() {
                    if result.is_none() {
                        result = Some(ast::Item::StructDef(structdef_ast.clone()));
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            ast::Item::PortDef(portdef_ast) => {
                if portdef_ast.name == item_id.name() {
                    if result.is_none() {
                        result = Some(ast::Item::PortDef(portdef_ast.clone()));
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            _ => (),
        }
    }

    result.ok_or_else(|| virdant_error!("Unknown item {item_id}"))
}

fn moddef_ast(db: &dyn AstQ, moddef_id: ModDefId) -> VirdantResult<Ast<ast::ModDef>> {
    let package_ast = db.package_ast(moddef_id.package())?;
    let mut result: Option<Ast<ast::ModDef>> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::ModDef(moddef_ast) => {
                if moddef_ast.name == moddef_id.name() {
                    if result.is_none() {
                        result = Some(moddef_ast.clone());
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            _ => (),
        }
    }

    result.ok_or_else(|| virdant_error!("Unknown moddef {moddef_id}"))
}

fn uniondef_ast(db: &dyn AstQ, uniontype_id: UnionDefId) -> VirdantResult<Ast<ast::UnionDef>> {
    let package_ast = db.package_ast(uniontype_id.package())?;
    let mut result: Option<Ast<ast::UnionDef>> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::UnionDef(uniondef_ast) => {
                if uniondef_ast.name == uniontype_id.name() {
                    if result.is_none() {
                        result = Some(uniondef_ast.clone());
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            _ => (),
        }
    }

    result.ok_or_else(|| virdant_error!("Unknown uniondef {uniontype_id}"))
}

fn structdef_ast(db: &dyn AstQ, structdef_id: StructDefId) -> VirdantResult<Ast<ast::StructDef>> {
    let package_ast = db.package_ast(structdef_id.package())?;
    let mut result: Option<Ast<ast::StructDef>> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::StructDef(structdef_ast) => {
                if structdef_ast.name == structdef_id.name() {
                    if result.is_none() {
                        result = Some(structdef_ast.clone());
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            _ => (),
        }
    }

    result.ok_or_else(|| virdant_error!("Unknown structdef {structdef_id}"))
}

fn portdef_ast(db: &dyn AstQ, portdef_id: PortDefId) -> VirdantResult<Ast<ast::PortDef>> {
    let package_ast = db.package_ast(portdef_id.package())?;
    let mut result: Option<Ast<ast::PortDef>> = None;

    for item in &package_ast.items {
        match item {
            ast::Item::PortDef(portdef_ast) => {
                if portdef_ast.name == portdef_id.name() {
                    if result.is_none() {
                        result = Some(portdef_ast.clone());
                    } else {
                        return Err(virdant_error!("Uh oh"));
                    }
                }
            },
            _ => (),
        }
    }

    result.ok_or_else(|| virdant_error!("Unknown portdef {portdef_id}"))
}

fn component_ast(db: &dyn AstQ, component_id: ComponentId) -> VirdantResult<Ast<ast::Component>> {
    let moddef_ast = db.moddef_ast(component_id.moddef())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(component) if component.name == component_id.name() => {
                return Ok(component.clone());
            },
            _ => (),
        }
    }
    Err(virdant_error!("No component: {component_id}"))
}

fn wire_ast(db: &dyn AstQ, moddef_id: ModDefId, path: Path) -> VirdantResult<Option<Ast<ast::Wire>>> {
    let moddef_ast = db.moddef_ast(moddef_id)?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Wire(wire) => {
                let ast::Wire(target, _, _) = wire.as_ref();
                if target == &path {
                    return Ok(Some(wire.clone()));
                }
            },
            ast::Decl::Component(component) => {
                if component.kind == ComponentKind::Incoming && component.name.as_path() == path {
                    return Ok(None);
                }
            },
            _ => (),
        }
    }
    Err(virdant_error!("No such wire: {}", path))
}
