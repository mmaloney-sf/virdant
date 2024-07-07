use std::collections::HashMap;
use crate::common::*;
use crate::sim::{Sim, SimBuilder};
use crate::db::{AstQ, TypecheckQ, TypedExpr};
use crate::db::Db;
use crate::ast;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Elab {
    pub moddef: Ident,
    pub submodules: HashMap<Ident, Elab>,
}


impl Elab {
    pub fn simulator(&self, db: &Db) -> Sim {
        let mut sim = Sim::new();
        let nonlocal_connects = HashMap::new();
        sim = self.add(db, "top".into(), sim, nonlocal_connects);
        sim.build()
    }

    fn add(&self, db: &Db, path: Path, mut sim: SimBuilder, nonlocal_connects: HashMap<Ident, Arc<TypedExpr>>) -> SimBuilder {
        let path_parts = path.parts();
        let base: Path = path_parts[path_parts.len() - 1].into();
        let components = db.moddef_components(self.moddef.clone()).unwrap();
        for component in &components {
            let full_path: Path = path.join(&component.name.as_path());
            let typ: Arc<Type> = db.moddef_component_type(self.moddef.clone(), component.name.clone()).unwrap().into();
            match component.kind {
                ast::SimpleComponentKind::Incoming => {
                    if let Some(expr) = nonlocal_connects.get(&component.name) {
                        sim = sim.add_simple_node(full_path, expr.clone(), true);
                    } else {
                        sim = sim.add_input_node(full_path, typ.clone());
                    }
                },
                ast::SimpleComponentKind::Outgoing => {
                    let expr: Arc<TypedExpr> = db.moddef_typecheck_wire(self.moddef.clone(), component.name.as_path()).unwrap();
                    sim = sim.add_simple_node(full_path, expr.clone(), false);
                },
                ast::SimpleComponentKind::Node => {
                    let expr: Arc<TypedExpr> = db.moddef_typecheck_wire(self.moddef.clone(), component.name.as_path()).unwrap();
                    sim = sim.add_simple_node(full_path, expr.clone(), false);
                },
                ast::SimpleComponentKind::Reg => {
                    let reset = None;
                    let expr: Arc<TypedExpr> = db.moddef_typecheck_wire(self.moddef.clone(), component.name.as_path()).unwrap();
                    sim = sim.add_reg_node(full_path, typ.clone(), component.clock.clone().unwrap(), reset, expr.clone());
                },
            }
        }

        /*
        for (name, submodule) in &self.submodules {
            let submodule_path = path.join(&name.as_path());
            let nonlocal_connects = self.moddef.nonlocal_connects_to(name.clone());
            sim = submodule.add(submodule_path, sim, nonlocal_connects);
        }
*/
        sim
    }
}
