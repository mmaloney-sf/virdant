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
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

pub fn compile_mlir(input: &str) -> VirdantResult<()> {
    let mut db = Db::default();
    db.set_source(Arc::new(input.to_string()));

    let package = db.package_hir()?;
    let mut stdout = std::io::stdout();
    package.mlir(&mut stdout).map_err(|_err| VirdantError::Unknown)?;
    Ok(())
}

pub fn compile_verilog(input: &str) -> VirdantResult<()> {
    let mut db = Db::default();
    db.set_source(Arc::new(input.to_string()));

    let mut stdout = std::io::stdout();
    db.verilog(&mut stdout).map_err(|_err| VirdantError::Unknown)?;
    Ok(())
}
