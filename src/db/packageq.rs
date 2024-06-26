use std::collections::HashMap;
use crate::common::*;
use super::TypecheckQ;
use crate::hir;
use crate::ast;
use crate::elab;

#[salsa::query_group(PackageQStorage)]
pub trait PackageQ: TypecheckQ {
    fn check_moddef(&self, moddef: Ident) -> VirdantResult<()>;
    fn check(&self) -> VirdantResult<()>;
    fn elaborate(&self, moddef: Ident) -> VirdantResult<elab::Elab>;

    fn package_hir(&self) -> VirdantResult<hir::Package>;
}

fn check(db: &dyn PackageQ) -> Result<(), VirdantError> {
    db.check_item_names_unique()?;
    db.check_submodule_moddefs_exist()?;
    db.check_no_submodule_cycles()?;

    let mut errors = ErrorReport::new();
    for moddef in &db.package_moddef_names()? {
        if let Err(err) = db.check_moddef(moddef.clone()) {
            errors.add(err);
        }
    }
    errors.check()
}

fn check_moddef(db: &dyn PackageQ, moddef: Ident) -> VirdantResult<()> {
    let mut errors = ErrorReport::new();
    for component in db.moddef_component_names(moddef.clone())? {
        let c = db.moddef_component_ast(moddef.clone(), component.clone())?;
        let connects = db.moddef_component_connects(moddef.clone(), component.clone())?;
        if c.kind == ast::ComponentKind::Incoming {
            if connects.len() > 0 {
                errors.add(VirdantError::Other(format!("connect for incoming {} in {}", component, moddef)));
            }
        } else {
            if connects.len() < 1 {
                errors.add(VirdantError::Other(format!("no connect for {} in {}", component, moddef)));
            } else if connects.len() > 1 {
                errors.add(VirdantError::Other(format!("multiple connects for {} in {}", component, moddef)));
            } else {
                if let Err(err) = db.typecheck_component(moddef.clone(), component.clone()) {
                    errors.add(err);
                }
            }
        }
    }
    errors.check()
}

fn elaborate(db: &dyn PackageQ, moddef: Ident) -> VirdantResult<elab::Elab> {
    let package = db.package_hir()?;
    package.elab(moddef.into())
}

fn package_hir(db: &dyn PackageQ) -> VirdantResult<hir::Package> {
    db.check()?;
    let mut moddefs = HashMap::new();

    for moddef_name in db.package_moddef_names()? {
        let moddef_hir = db.moddef_hir_typed(moddef_name.clone())?;
        moddefs.insert(moddef_name.clone(), moddef_hir.into());
    }

    Ok(hir::Package {
        moddefs,
    })
}
