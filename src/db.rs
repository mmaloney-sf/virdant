mod astq;
mod structureq;
mod typecheckq;
mod packageq;

pub use astq::AstQ;
pub use structureq::StructureQ;
pub use typecheckq::TypecheckQ;
pub use packageq::PackageQ;

use std::sync::Arc;
use crate::common::*;

#[salsa::database(
    astq::AstQStorage,
    structureq::StructureQStorage,
    typecheckq::TypecheckQStorage,
    packageq::PackageQStorage,
)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

pub fn compile(input: &str) -> VirdantResult<()> {
    let mut db = Database::default();
    db.set_source(Arc::new(input.to_string()));

    let package = db.package_hir()?;
    let mut stdout = std::io::stdout();
    package.mlir(&mut stdout).map_err(|_err| VirdantError::Unknown)?;
    Ok(())
}
