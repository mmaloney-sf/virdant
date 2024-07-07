use std::collections::{HashSet, HashMap};
use crate::ast::SimpleComponentKind;
use crate::common::*;
use crate::ast;
pub use super::AstQ;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: AstQ {
    fn package_item_names(&self) -> VirdantResult<Vec<Ident>>;
    fn package_moddef_names(&self) -> VirdantResult<Vec<Ident>>;
    fn moddef_component_names(&self, moddef: Ident) -> VirdantResult<Vec<Ident>>;
    fn moddef_names(&self, moddef: Ident) -> VirdantResult<Vec<Ident>>;
    fn moddef_port_names(&self, moddef: Ident) -> VirdantResult<Vec<Ident>>;

    fn moddef_required_targets(&self, moddef: Ident) -> VirdantResult<Vec<Path>>;
    fn moddef_wire_targets(&self, moddef: Ident) -> VirdantResult<Vec<Path>>;

    fn check_item_names_unique(&self) -> VirdantResult<()>;
    fn check_moddef_component_names_unique(&self, moddef: Ident) -> VirdantResult<()>;
    fn check_moddef_wire_targets_unique(&self, moddef: Ident) -> VirdantResult<()>;

    fn check_submodule_moddefs_exist(&self) -> VirdantResult<()>;
    fn check_no_submodule_cycles(&self) -> VirdantResult<()>;
}

fn package_item_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
            ast::Item::StructTypeDef(structtypedef) => result.push(structtypedef.name.clone()),
        }
    }
    Ok(result)
}

fn package_moddef_names(db: &dyn StructureQ) -> Result<Vec<Ident>, VirdantError> {
    let package = db.package_ast()?;
    let mut result = vec![];
    for item in &package.items {
        match item {
            ast::Item::ModDef(moddef) => result.push(moddef.name.clone()),
            ast::Item::StructTypeDef(_) => (),
        }
    }
    Ok(result)
}

fn moddef_component_names(db: &dyn StructureQ, moddef: Ident) -> Result<Vec<Ident>, VirdantError> {
    let moddef = db.moddef_ast(moddef)?;
    let mut result = vec![];
    for decl in moddef.decls {
        match decl {
            ast::Decl::SimpleComponent(component) => result.push(component.name.clone()),
            ast::Decl::Submodule(_submodule) => (),
            ast::Decl::Wire(_connect) => (),
        }
    }
    Ok(result)
}

fn moddef_names(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<Vec<Ident>> {
    let mut results = vec![];

    let component_asts = db.moddef_components(moddef.clone())?;
    results.extend(component_asts.iter().map(|component| component.name.clone()).collect::<Vec<_>>());

    let submodule_asts = db.moddef_submodules(moddef)?;
    results.extend(submodule_asts.iter().map(|submodule| submodule.name.clone()).collect::<Vec<_>>());

    Ok(results)
}

fn moddef_port_names(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<Vec<Ident>> {
    let mut results = vec![];

    let component_asts = db.moddef_components(moddef.clone())?;
    results.extend(component_asts
        .iter()
        .filter(|component| component.kind == SimpleComponentKind::Incoming || component.kind == SimpleComponentKind::Outgoing)
        .map(|component| component.name.clone())
        .collect::<Vec<_>>());

    Ok(results)
}

fn moddef_required_targets(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<Vec<Path>> {
    let mut result: Vec<Path> = vec![];
    let moddef_ast = db.moddef_ast(moddef.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(component) => {
                if component.kind != SimpleComponentKind::Incoming {
                    result.push(component.name.clone().into());
                }
            },
            ast::Decl::Submodule(submodule) => {
                let submodule_moddef_ast = db.moddef_ast(submodule.moddef.clone())?;
                for decl in &submodule_moddef_ast.decls {
                    if let ast::Decl::SimpleComponent(component) = decl {
                        if component.kind == SimpleComponentKind::Incoming {
                            let submodule_path: Path = submodule.name.clone().into();
                            let component_path: Path = component.name.clone().into();
                            result.push(submodule_path.join(&component_path));
                        }
                    }
                }
            },
            ast::Decl::Wire(_) => (),
        }

    }

    Ok(result)
}

fn moddef_wire_targets(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<Vec<Path>> {
    let mut result: Vec<Path> = vec![];
    let moddef_ast = db.moddef_ast(moddef.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Wire(ast::Wire(target, _wire_type, _expr)) => result.push(target.clone()),
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

fn check_moddef_component_names_unique(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    let mut item_names = HashSet::new();

    for name in db.moddef_names(moddef.clone())? {
        if !item_names.insert(name.clone()) {
            errors.add(VirdantError::Other(format!("Duplicate name: {name} in module {moddef}", moddef=moddef.clone())));
        }
    }

    errors.check()
}


fn check_moddef_wire_targets_unique(db: &dyn StructureQ, moddef: Ident) -> VirdantResult<()> {
    todo!()
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
