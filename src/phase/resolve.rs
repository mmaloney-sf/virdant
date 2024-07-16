use std::collections::HashSet;

use crate::common::*;
use crate::ast;
use super::*;

#[salsa::query_group(ResolveQStorage)]
pub trait ResolveQ: astq::AstQ {
    fn resolve_package2(&self, package_name: Ident) -> VirdantResult<PackageId>;
}

fn resolve_package2(db: &dyn ResolveQ, package_name: Ident) -> VirdantResult<PackageId> {
    for package in db.packages() {
        if package.fqname() == package_name.as_path() {
            return Ok(package);
        }
    }
    Err(VirdantError::Unknown)
}
