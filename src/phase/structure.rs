use crate::ast::ComponentKind;
use crate::context::Context;
use crate::{ast, common::*};
use super::*;

use super::typecheck::TypedExpr;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: check::CheckQ {
    fn structure_moddef(&self, moddef: ModDefId) -> VirdantResult<ModDef>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModDef {
    id: ModDefId,
    components: Vec<Component>,
    submodules: Vec<Submodule>,
    ext: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Component {
    id: ComponentId,
    typ: Type,
    kind: ast::ComponentKind,
    driver: Option<Arc<TypedExpr>>,
    clock: Option<Arc<TypedExpr>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Submodule {
    id: ComponentId,
    moddef_id: ModDefId,
    drivers: HashMap<Path, Arc<TypedExpr>>,
}


impl ModDef {
    pub fn id(&self) -> ModDefId {
        self.id.clone()
    }

    pub fn components(&self) -> Vec<Component> {
        self.components.clone()
    }

    pub fn ports(&self) -> Vec<Component> {
        self.components().into_iter().filter(|el| el.is_port()).collect()
    }

    pub fn submodules(&self) -> Vec<Submodule> {
        self.submodules.clone()
    }

    pub fn is_ext(&self) -> bool {
        self.ext
    }
}

impl Component {
    pub fn id(&self) -> ComponentId {
        self.id.clone()
    }

    pub fn typ(&self) -> Type {
        self.typ.clone()
    }

    pub fn is_port(&self) -> bool {
        self.is_incoming() || self.is_outgoing()
    }

    pub fn is_reg(&self) -> bool {
        self.kind == ComponentKind::Reg
    }

    pub fn is_incoming(&self) -> bool {
        self.kind == ComponentKind::Incoming
    }

    pub fn is_outgoing(&self) -> bool {
        self.kind == ComponentKind::Outgoing
    }

    pub fn is_node(&self) -> bool {
        self.kind == ComponentKind::Node
    }

    pub fn driver(&self) -> Option<Arc<TypedExpr>> {
        self.driver.clone()
    }

    pub fn clock(&self) -> Option<Arc<TypedExpr>> {
        self.clock.clone()
    }
}

impl Submodule {
    pub fn id(&self) -> ComponentId {
        self.id.clone()
    }

    pub fn moddef(&self) -> ModDefId {
        self.moddef_id.clone()
    }

    pub fn driver_for(&self, path_id: Path) -> Arc<TypedExpr> {
        self.drivers.get(&path_id).expect(&format!("No driver for {path_id}")).clone()
    }
}

fn structure_moddef(db: &dyn StructureQ, moddef_id: ModDefId) -> VirdantResult<ModDef> {
    db.check()?;

    let mut components = vec![];
    let mut submodules = vec![];

    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(component) => {
                let typ = db.resolve_typ(component.typ.clone(), moddef_id.package())?;

                let driver = if !moddef_ast.ext {
                    let wire = db.wire_ast(moddef_id.clone(), component.name.as_path())?;

                    match wire {
                        Some(ast::Wire(_target, _wire_type, expr)) => Some(db.typecheck_expr(moddef_id.clone(), expr, typ.clone(), Context::empty())?),
                        None => None,
                    }
                } else {
                    None
                };

                let clock = if component.kind == ComponentKind::Reg {
                    let clock_expr = component.clock.clone().unwrap();
                    let typed_expr = db.typecheck_expr(moddef_id.clone(), clock_expr, Type::Clock, Context::empty())?;
                    Some(typed_expr)
                } else {
                    None
                };

                components.push(
                    Component {
                        id: ComponentId::from_ident(moddef_id.clone(), component.name.clone()),
                        typ,
                        kind: component.kind.clone(),
                        driver,
                        clock,
                    }
                )
            },
            ast::Decl::Submodule(submodule) => {
                eprintln!("--------------------------------------------------------------------------------");
                eprintln!("Submodule in Structure of module {moddef_id}: {submodule:?}");
                let submodule_moddef_id = db.moddef(submodule.moddef.clone(), moddef_id.package())?;

                let mut drivers = HashMap::new();

                let submodule_ast = db.moddef_ast(submodule_moddef_id.clone())?;
                let mut incomings = HashMap::new();

                for decl in &submodule_ast.decls {
                    match decl {
                        ast::Decl::Component(component) if component.kind == ComponentKind::Incoming => {
                            let target_path = submodule.name.as_path().join(&component.name.as_path());
                            let component_id = db.resolve_component(moddef_id.clone(), target_path.clone())?;
                            eprintln!("component_id = {component_id}");
                            incomings.insert(target_path, component_id);
                        },
                        _ => (),
                    }
                }

                for decl in &moddef_ast.decls {
                    if let ast::Decl::Wire(ast::Wire(target, _wire_type, expr)) = decl {
                        eprintln!("looking at wire targeting: {target}");
                        if incomings.contains_key(target) {
                            eprintln!("target = {target}");
                            let component_id = db.resolve_component(moddef_id.clone(), target.clone())?;
                            eprintln!("component_id = {component_id}");
                            let typ = db.component_typ(component_id)?;
                            let typed_expr = db.typecheck_expr(moddef_id.clone(), expr.clone(), typ.clone(), Context::empty())?;
                            drivers.insert(target.clone(), typed_expr);
                        }
                    }
                }
                dbg!(&drivers);
                submodules.push(
                    Submodule {
                        id: ComponentId::from_ident(moddef_id.clone(), submodule.name.clone()),
                        moddef_id: submodule_moddef_id,
                        drivers,
                    }
                );
                eprintln!("DONE");
                eprintln!("--------------------------------------------------------------------------------");
            },
            ast::Decl::Port(_port) => todo!(),
            ast::Decl::Wire(_wire) => (),
        }
    }

    let moddef = ModDef {
        id: moddef_id,
        components,
        submodules,
        ext: moddef_ast.ext,
    };
    Ok(moddef)
}

    /*
fn moddef_components(db: &dyn ItemStructureQ, moddef: ModDefId) -> VirdantResult<Vec<ComponentId>> {
    let mut components: Vec<ComponentId> = vec![];
    let mut errors = ErrorReport::new();
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(simplecomponent) => {
                let component = ComponentId::from(Path::from(moddef.clone()).join(&simplecomponent.name.as_path()));

                if components.contains(&component) {
                    errors.add(VirdantError::Other(format!("Moddef contains a duplicate component: {moddef} {}", simplecomponent.name)));
                } else {
                    components.push(component);
                }
            },
            ast::Decl::Submodule(submodule) => {
                let component = ComponentId::from(Path::from(moddef.clone()).join(&submodule.name.as_path()));

                if components.contains(&component) {
                    errors.add(VirdantError::Other(format!("Moddef contains a duplicate component: {moddef} {}", submodule.name)));
                } else {
                    components.push(component);
                }
            },
            ast::Decl::Wire(_) => (),
            ast::Decl::Port(_) => todo!(),
        }
    }
    errors.check()?;
    Ok(components)
}

fn uniondef_alts(db: &dyn ItemStructureQ, uniondef: UnionDefId) -> VirdantResult<Vec<AltId>> {
    let mut alts: Vec<AltId> = vec![];
    let mut errors = ErrorReport::new();
    let uniondef_ast = db.uniondef_ast(uniondef.clone())?;
    for ast::Alt(name, _typs) in &uniondef_ast.alts {
        let alt = AltId::from(uniondef.fqname().join(&name.as_path()));
        if alts.contains(&alt) {
            errors.add(VirdantError::Other(format!("Uniondef contains a duplicate alt: {uniondef} {name}")));
        } else {
            alts.push(alt);
        }
    }
    errors.check()?;
    Ok(alts)
}
    */
