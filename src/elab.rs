use std::collections::HashMap;
use crate::common::*;
use crate::hir;
use crate::sim::{Sim, SimBuilder};
use crate::types::Type;
use crate::context::Context;

pub struct Elab {
    pub moddef: Arc<hir::ModDef>,
    pub submodules: HashMap<Ident, Elab>,
}


impl Elab {
    pub fn simulator(&self) -> Sim {
        let mut sim = Sim::new();
        sim = self.add("top".into(), sim);
        sim.build()
    }

    fn add(&self, path: Path, mut sim: SimBuilder) -> SimBuilder {
        for component in &self.moddef.components {
            let full_path: Path = path.join(&component.name().as_path());
            match component {
                hir::Component::Incoming(_name, typ) => {
                    sim = sim.add_input_node(full_path, typ.clone());
                }
                hir::Component::Outgoing(_name, typ, expr) => {
                    sim = sim.add_simple_node(full_path, expr.clone());
                },
                hir::Component::Wire(_name, typ, expr) => {
                    sim = sim.add_simple_node(full_path, expr.clone());
                },
                hir::Component::Reg(_name, typ, clk, expr) => {
                    let reset = None;
                    sim = sim.add_reg_node(full_path, typ.clone(), clk.clone(), reset, expr.clone());
                },
            }
        }

        for (name, submodule) in &self.submodules {
            let submodule_path = path.join(&name.as_path());
            sim = submodule.add(submodule_path, sim);
        }
        sim
    }
}