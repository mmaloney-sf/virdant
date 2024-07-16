use structure::typecheck::Referent;

use crate::ast::ComponentKind;
use crate::{ast, common::*};
use super::*;

use super::typecheck::TypedExpr;

#[salsa::query_group(StructureQStorage)]
pub trait StructureQ: typecheck::TypecheckQ {
    fn moddef(&self, moddef: ModDefId) -> VirdantResult<ModDef>;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModDef {
    id: ModDefId,
    elements: Vec<Element>,
    submodules: Vec<Submodule>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Element {
    id: ComponentId,
    typ: Type,
    kind: ast::ComponentKind,
    driver: Arc<TypedExpr>,
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

    pub fn elements(&self) -> Vec<Element> {
        self.elements.clone()
    }

    pub fn ports(&self) -> Vec<Element> {
        self.elements().into_iter().filter(|el| el.is_port()).collect()
    }

    pub fn internals(&self) -> Vec<Element> {
        self.elements().into_iter().filter(|el| el.is_internal()).collect()
    }

    pub fn submodules(&self) -> Vec<Submodule> {
        self.submodules.clone()
    }
}

impl Element {
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

    pub fn driver(&self) -> Arc<TypedExpr> {
        self.driver.clone()
    }

    pub fn clock(&self) -> Option<Arc<TypedExpr>> {
        if self.is_reg() {
            eprintln!("HACK on clock {}:{}", file!(), line!());
            Some(TypedExpr::Reference(Type::Clock, Referent::Element(Path::from("clock").into())).into())
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

fn moddef(db: &dyn StructureQ, moddef: ModDefId) -> VirdantResult<ModDef> {
    db.typecheck(moddef.clone())?;
    todo!()
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
