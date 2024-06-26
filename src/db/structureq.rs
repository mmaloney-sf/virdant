use std::collections::{HashSet, HashMap};
use crate::common::*;
use crate::hir;
use crate::ast;
use crate::types::Type;
pub use super::AstQ;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: AstQ {
    fn moddef_hir(&self, moddef: Ident) -> VirdantResult<hir::ModDef>;
    fn moddef_submodules(&self, moddef: Ident) -> VirdantResult<Vec<hir::Submodule>>;
    fn moddef_component_hir(&self, moddef: Ident, component: Ident) -> VirdantResult<hir::Component>;

    fn package_item_names(&self) -> VirdantResult<Vec<Ident>>;
    fn package_moddef_names(&self) -> VirdantResult<Vec<Ident>>;
    fn moddef_component_names(&self, moddef: Ident) -> VirdantResult<Vec<Ident>>;
    fn moddef_component_connects(&self, moddef: Ident, component: Ident) -> VirdantResult<Vec<ast::InlineConnect>>;
    fn moddef_submodule_connects(&self, moddef: Ident, submodule: Ident) -> VirdantResult<Vec<ast::Connect>>;
    fn moddef_submodule_moddef(&self, moddef: Ident, submodule: Ident) -> VirdantResult<Ident>;

    fn check_item_names_unique(&self) -> VirdantResult<()>;
    fn check_submodule_moddefs_exist(&self) -> VirdantResult<()>;
    fn check_no_submodule_cycles(&self) -> VirdantResult<()>;
}

fn moddef_component_hir(db: &dyn StructureQ, moddef: Ident, component: Ident) -> VirdantResult<hir::Component> {
    let c = db.moddef_component_ast(moddef.clone(), component.clone())?;
    let typ = Type::from_ast(&c.typ);

    let connects = db.moddef_component_connects(moddef.clone(), component.clone())?;

    Ok(match c.kind {
        ast::ComponentKind::Incoming => hir::Component::Incoming(c.name.clone(), typ),
        ast::ComponentKind::Outgoing => {
            if connects.len() == 0 {
                return Err(VirdantError::Other(format!("No connections to {}", c.name.clone())));
            }
            let ast::InlineConnect(_connect_type, expr) = connects[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            hir::Component::Outgoing(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Wire => {
            if connects.len() == 0 {
                return Err(VirdantError::Other(format!("No connections to {}", c.name.clone())));
            }
            let ast::InlineConnect(_connect_type, expr) = connects[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            hir::Component::Wire(c.name.clone(), Type::from_ast(&c.typ), expr)
        },
        ast::ComponentKind::Reg => {
            if connects.len() == 0 {
                return Err(VirdantError::Other(format!("No connections to {}", c.name.clone())));
            }
            let ast::InlineConnect(_connect_type, expr) = connects[0].clone();
            let expr = hir::Expr::from_ast(&expr);
            let clock = c.clock.ok_or_else(|| VirdantError::Other(format!("No \"on\" clause for reg")))?;
            hir::Component::Reg(c.name.clone(), Type::from_ast(&c.typ), clock, expr)
        },
    })
}

fn moddef_hir(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<hir::ModDef> {
    let mut components: Vec<hir::Component> = vec![];
    let mut submodules: Vec<hir::Submodule> = vec![];
    let mut connects: Vec<hir::Connect> = vec![];

    for decl in db.moddef_ast(moddef.clone())?.decls {
        match decl {
            ast::Decl::Submodule(m) => submodules.push(
                hir::Submodule {
                    name: m.name,
                    moddef: m.moddef,
                }
            ),
            ast::Decl::Connect(ast::Connect(target, connect_type, expr_ast)) => {
                let expr = hir::Expr::from_ast(&expr_ast);
                connects.push(hir::Connect(target.clone(), connect_type, expr));
            },
            _ => (),
        }
    }

    for component_name in db.moddef_component_names(moddef.clone())? {
        let component = db.moddef_component_hir(moddef.clone(), component_name)?;
        components.push(component);
    }

    Ok(hir::ModDef {
        name: moddef.clone(),
        components,
        submodules,
        connects,
    })
}

fn package_item_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
            ast::Item::StructTypeDef(structtypedef) => result.push(structtypedef.name.clone()),
            ast::Item::PortDef(port) => result.push(port.name.clone()),
        }
    }
    Ok(result)
}

fn moddef_component_names(db: &dyn StructureQ, moddef: Ident) -> Result<Vec<Ident>, VirdantError> {
    let moddef = db.moddef_ast(moddef)?;
    let mut result = vec![];
    for decl in moddef.decls {
        match decl {
            ast::Decl::Component(component) => result.push(component.name.clone()),
            ast::Decl::Submodule(_submodule) => (),
            ast::Decl::Connect(_connect) => (),
            ast::Decl::Port(_port) => (),
        }
    }
    Ok(result)
}

fn moddef_submodules(db: &dyn StructureQ, moddef: Ident) -> Result<Vec<hir::Submodule>, VirdantError> {
    let moddef_hir = db.moddef_hir(moddef.clone())?;
    Ok(moddef_hir.submodules.iter().cloned().collect())
}

fn package_moddef_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
            ast::Item::StructTypeDef(_typedef) => (),
            ast::Item::PortDef(_interface) => (),
        }
    }
    Ok(result)
}

fn moddef_component_connects(db: &dyn StructureQ, moddef: Ident, component: Ident) -> Result<Vec<ast::InlineConnect>, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut result = vec![];

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(c) if c.name == component => {
                if let Some(connect) = &c.connect {
                    result.push(connect.clone());
                }
            },
            ast::Decl::Connect(ast::Connect(target, connect_type, expr)) if target == &component.as_path() => {
                result.push(ast::InlineConnect(*connect_type, expr.clone()));
            },
            _ => (),
        }
    }
    Ok(result)
}

fn moddef_submodule_connects(db: &dyn StructureQ, moddef: Ident, submodule: Ident) -> Result<Vec<ast::Connect>, VirdantError> {
    let moddef_ast = db.moddef_ast(moddef)?;
    let mut result = vec![];

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Connect(ast::Connect(target, connect_type, expr)) if target.is_nonlocal() => {
                if target.parts()[0] == submodule.as_str() {
                    result.push(ast::Connect(target.clone(), *connect_type, expr.clone()));
                }

            },
            _ => (),
        }
    }
    Ok(result)
}

fn check_item_names_unique(db: &dyn StructureQ) -> Result<(), VirdantError> {
    let mut errors = ErrorReport::new();
    let mut item_names = HashSet::new();

    for name in db.package_item_names()? {
        if !item_names.insert(name.clone()) {
            errors.add(VirdantError::Other(format!("Duplicate item: {name}")));
        }
    }

    errors.check()
}

fn check_submodule_moddefs_exist(db: &dyn StructureQ) -> Result<(), VirdantError> {
    let mut errors = ErrorReport::new();

    let moddef_names: Vec<Ident> = db.package_moddef_names()?;
    for moddef_name in &moddef_names {
        for submodule in &db.moddef_submodules(moddef_name.clone())? {
            if !moddef_names.contains(&submodule.moddef) {
                let submoddef_name = &submodule.moddef;
                let msg = format!("Module contains an undefined submodule: {moddef_name} contains unknown {submoddef_name}");
                errors.add(VirdantError::Other(msg));
            }
        }
    }

    errors.check()
}

fn check_no_submodule_cycles(db: &dyn StructureQ) -> Result<(), VirdantError> {
    let mut errors = ErrorReport::new();
    let moddef_names: Vec<Ident> = db.package_moddef_names()?;

    let mut depends: HashMap<Ident, Vec<Ident>> = HashMap::new();

    for moddef_name in &moddef_names {
        let submodules = db.moddef_submodules(moddef_name.clone())?;
        depends.insert(moddef_name.clone(), submodules.into_iter().map(|s| s.moddef).collect());
    }

    for cycle in find_cycles(&depends) {
        let moddef = &cycle[0];
        let cycle_str = cycle.iter().map(|moddef| moddef.as_str()).collect::<Vec<_>>().join(" contains ");
        errors.add(VirdantError::Other(format!("Module contains itself: {cycle_str} contains {moddef}")));
    }

    errors.check()
}

fn moddef_submodule_moddef(db: &dyn StructureQ, moddef: Ident, submodule: Ident) -> VirdantResult<Ident> {
    for decl in db.moddef_ast(moddef.clone())?.decls {
        match decl {
            ast::Decl::Submodule(m) => return Ok(m.moddef),
            _ => (),
        }
    }
    Err(VirdantError::Other(format!("Unknown submodule: {submodule} in {moddef}")))
}

fn find_cycles(graph: &HashMap<Ident, Vec<Ident>>) -> Vec<Vec<Ident>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            dfs(node.clone(), graph, &mut visited, &mut stack, &mut cycles);
        }
    }

    cycles
}

fn dfs(
    node: Ident,
    graph: &HashMap<Ident, Vec<Ident>>,
    visited: &mut HashSet<Ident>,
    stack: &mut Vec<Ident>,
    cycles: &mut Vec<Vec<Ident>>,
) {
    visited.insert(node.clone());
    stack.push(node.clone());

    if let Some(neighbors) = graph.get(&node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs(neighbor.clone(), graph, visited, stack, cycles);
            } else if stack.contains(&neighbor) {
                // Found a cycle
                let cycle_start = stack.iter().position(|x| x == neighbor).unwrap();
                let cycle: Vec<Ident> = stack[cycle_start..].iter().cloned().collect();
                cycles.push(cycle);
            }
        }
    }

    stack.pop();
}
