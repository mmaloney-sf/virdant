use structure::typecheck::Referent;

use crate::ast::ComponentKind;
use crate::context::Context;
use crate::{ast, common::*};
use super::*;

use super::typecheck::TypedExpr;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: check::CheckQ {
    fn moddef(&self, moddef: ModDefId) -> VirdantResult<ModDef>;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModDef {
    id: ModDefId,
    components: Vec<Component>,
    submodules: Vec<Submodule>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Component {
    id: ComponentId,
    typ: Type,
    kind: ast::ComponentKind,
    driver: Option<Arc<TypedExpr>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Submodule {
    id: ComponentId,
    moddef_id: ModDefId,
}


impl ModDef {
    pub fn id(&self) -> ModDefId {
        self.id.clone()
    }

    pub fn elements(&self) -> Vec<Component> {
        self.components.clone()
    }

    pub fn ports(&self) -> Vec<Component> {
        self.elements().into_iter().filter(|el| el.is_port()).collect()
    }

    pub fn internals(&self) -> Vec<Component> {
        self.elements().into_iter().filter(|el| el.is_internal()).collect()
    }

    pub fn submodules(&self) -> Vec<Submodule> {
        self.submodules.clone()
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

    pub fn is_internal(&self) -> bool {
        self.is_reg() || self.is_node()
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
        if self.is_reg() {
            eprintln!("HACK on clock {}:{}", file!(), line!());

            let component_id: ComponentId = self.id();
            Some(TypedExpr::Reference(Type::Clock, Referent::Component(component_id)).into())
        } else {
            None
        }
    }
}

impl Submodule {
    pub fn id(&self) -> ComponentId {
        self.id.clone()
    }

    pub fn moddef(&self) -> ModDefId {
        self.moddef_id.clone()
    }
}

fn moddef(db: &dyn StructureQ, moddef_id: ModDefId) -> VirdantResult<ModDef> {
    db.check()?;

    let mut components = vec![];
    let mut submodules = vec![];

    let moddef_ast = db.moddef_ast(moddef_id.clone())?;

    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::Component(component) => {
                let typ = db.resolve_typ(component.typ.clone(), moddef_id.package())?;
                let component_path_id = db.resolve_path(moddef_id.clone(), component.name.as_path())?;

                let wire = db.wire_ast(component_path_id)?;

                let driver = match wire {
                    Some(ast::Wire(_target, _wire_type, expr)) => Some(db.typecheck_expr(moddef_id.clone(), expr, typ.clone(), Context::empty())?),
                    None => None,
                };

                components.push(
                    Component {
                        id: ComponentId::from_ident(moddef_id.clone(), component.name.clone()),
                        typ,
                        kind: component.kind.clone(),
                        driver,
                    }
                )
            },
            ast::Decl::Submodule(submodule) => todo!(),
            ast::Decl::Port(port) => todo!(),
            ast::Decl::Wire(_wire) => (),
        }
    }

    let moddef = ModDef {
        id: moddef_id,
        components,
        submodules,
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
