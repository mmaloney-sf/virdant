mod check;
mod coalesceconnects;

use std::collections::HashSet;
use crate::common::*;
use super::*;

pub trait Pass {
    fn run(&self, package: Package) -> Result<Package, VirdantError>;
}

impl Package {
    pub fn run_passes(mut self) -> Result<Self, VirdantError> {
        let passes: Vec<Box<dyn Pass>> = vec![
            Box::new(check::Check),
            Box::new(coalesceconnects::CoalesceConnects),
        ];
        for pass in passes {
            self = pass.run(self)?;
        }
        Ok(self)
    }
}
