mod astq;
mod structureq;
mod typecheckq;
mod packageq;

pub use astq::AstQ;
pub use structureq::StructureQ;
pub use typecheckq::TypecheckQ;
pub use packageq::PackageQ;

use std::sync::Arc;
use crate::hir;
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

pub fn check_module(input: &str) -> VirdantResult<hir::Package> {
    let mut db = Database::default();
    db.set_source(Arc::new(input.to_string()));
    Ok(db.package_hir()?)
}

#[test]
fn test_checker() {
    let mut db = Database::default();
    db.set_source(Arc::new("
        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];
            reg r : Word[8] on clk <= in;
            out := in->add(1w8);
            submodule foo of Foo;
        }

        module Foo {
            wire w : Word[8] := 0;
        }
    ".to_string()));

    db.check().unwrap();
}
