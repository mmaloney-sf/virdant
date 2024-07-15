use crate::ast;
use crate::common::*;
use super::*;

#[salsa::query_group(ItemStructureQStorage)]
pub trait ItemStructureQ: astq::AstQ {
    fn moddef_components(&self, moddef: ModDefId) -> VirdantResult<Vec<ComponentId>>;
    fn uniondef_alts(&self, uniondef: UnionDefId) -> VirdantResult<Vec<AltId>>;
}

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
