use crate::ast;
use crate::common::*;
use super::*;

#[salsa::query_group(ItemStructureQStorage)]
pub trait ItemStructureQ: astq::AstQ {
    fn moddef_components(&self, moddef: ModDef) -> VirdantResult<Vec<Component>>;
    fn uniondef_alts(&self, uniondef: UnionDef) -> VirdantResult<Vec<Alt>>;

}

fn moddef_components(db: &dyn ItemStructureQ, moddef: ModDef) -> VirdantResult<Vec<Component>> {
    let mut components: Vec<Component> = vec![];
    let mut errors = ErrorReport::new();
    let moddef_ast = db.moddef_ast(moddef.clone())?;
    for decl in &moddef_ast.decls {
        match decl {
            ast::Decl::SimpleComponent(simplecomponent) => {
                let component = Component::from(Path::from(moddef.clone()).join(&simplecomponent.name.as_path()));

                if components.contains(&component) {
                    errors.add(VirdantError::Other(format!("Moddef contains a duplicate component: {moddef} {}", simplecomponent.name)));
                } else {
                    components.push(component);
                }
            },
            ast::Decl::Submodule(submodule) => {
                let component = Component::from(Path::from(moddef.clone()).join(&submodule.name.as_path()));

                if components.contains(&component) {
                    errors.add(VirdantError::Other(format!("Moddef contains a duplicate component: {moddef} {}", submodule.name)));
                } else {
                    components.push(component);
                }
            },
            ast::Decl::Wire(_) => (),
        }
    }
    errors.check()?;
    Ok(components)
}

fn uniondef_alts(db: &dyn ItemStructureQ, uniondef: UnionDef) -> VirdantResult<Vec<Alt>> {
    let mut alts: Vec<Alt> = vec![];
    let mut errors = ErrorReport::new();
    let uniondef_ast = db.uniondef_ast(uniondef.clone())?;
    for ast::Alt(name, _typs) in &uniondef_ast.alts {
        let alt = Alt::from(uniondef.fqname().join(&name.as_path()));
        if alts.contains(&alt) {
            errors.add(VirdantError::Other(format!("Uniondef contains a duplicate alt: {uniondef} {name}")));
        } else {
            alts.push(alt);
        }
    }
    errors.check()?;
    Ok(alts)
}
