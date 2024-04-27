use std::collections::HashMap;
use super::*;

type CellId = usize;
type ClockId = usize;
type ResetId = usize;

#[derive(Debug, Clone)]
pub struct Sim {
    nodes: Vec<Node>,
    cells: Vec<Value>,
    events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct SimBuilder {
    sim: Sim,
}

impl SimBuilder {
    pub fn build(mut self) -> Sim {
        self.patch_sensitivity_lists();
        self.initialize_constants();
        self.sim.flow();
        self.sim
    }

    pub fn add_simple_node(mut self, path: Path, typ: Type, expr: Expr) -> Self {
        let cell_id = self.sim.cells.len();

        let update = Comb {
            rel: path.parent(),
            expr,
            sensitivities: vec![],
        };

        let node = Node::Simple {
            cell_id,
            path: path.clone(),
            typ: typ.clone(),
            update,
        };

        self.sim.nodes.push(node);
        self.sim.cells.push(Value::X(typ.clone()));
        self
    }

    pub fn add_reg_node(mut self, path: Path, typ: Type, reset: Option<Value>, expr: Expr) -> Self {
        let set_cell_id = self.sim.cells.len();
        let val_cell_id = self.sim.cells.len() + 1;

        let update = Comb {
            rel: path.parent(),
            expr,
            sensitivities: vec![],
        };

        let node = Node::Reg {
            set_cell_id,
            val_cell_id,
            path: path.clone(),
            typ: typ.clone(),
            update,
            reset,
        };

        self.sim.nodes.push(node);
        self.sim.cells.push(Value::X(typ.clone()));
        self.sim.cells.push(Value::X(typ.clone()));
        self
    }

    fn patch_sensitivity_lists(&mut self) {
        let mut path_read_cell_ids = HashMap::new();
        for node in &self.sim.nodes {
            path_read_cell_ids.insert(node.path().clone(), node.read_cell_id());
        }

        for node in &mut self.sim.nodes {
            let update = node.update_mut();
            let sensitivities: Vec<CellId> = update
                .expr
                .references()
                .iter()
                .map(|path| {
                    let full_path = update.rel.join(path);
                    path_read_cell_ids[&full_path]
                })
                .collect();
            update.sensitivities = sensitivities;
        }
    }

    fn initialize_constants(&mut self) {
        for i in 0..self.sim.nodes.len() {
            let node = &self.sim.nodes[i];
            let update = node.update().clone();
            if update.is_constant() {
                let value = self.sim.eval(&update);
                let cell_id = node.target_cell_id();
                self.sim.update_cell(cell_id, value);
            }
        }
    }
}

impl Sim {
    pub fn new() -> SimBuilder {
        let sim = Sim {
            nodes: vec![],
            cells: vec![],
            events: vec![],
        };
        SimBuilder {
            sim,
        }
    }

    fn flow(&mut self) {
        while let Some(event) = self.events.pop() {
            for node in &self.nodes.clone() {
                match (event, node) {
                    (Event::CellUpdated(updated_cell_id), _) => {
                        let update = node.update();
                        if update.is_sensitive_to(updated_cell_id) {
                            let value = self.eval(update);
                            let cell_id = node.target_cell_id();
                            self.update_cell(cell_id, value);
                        }
                    },
                    (Event::Clock(_clock_id), Node::Reg { set_cell_id, val_cell_id, .. }) => {
                        let value = self.get_cell(*set_cell_id).clone();
                        self.update_cell(*val_cell_id, value);
                    },
                    (Event::Reset(_reset_id), Node::Reg { val_cell_id, reset, .. }) => {
                        if let Some(value) =  reset {
                            self.update_cell(*val_cell_id, value.clone());
                        }
                    },
                    _ => (),

                }
            }
        }
    }

    fn update_cell(&mut self, cell_id: CellId, value: Value) {
        self.cells[cell_id] = value;
        self.events.push(Event::CellUpdated(cell_id));
    }

    // keep around for debugging
    #[allow(dead_code)]
    fn event_name(&self, event: &Event) -> String {
        match event {
            Event::CellUpdated(cell_id) => format!("updated #{}", self.cell_name(*cell_id)),
            Event::Clock(clock_id) => format!("clock #{clock_id}"),
            Event::Reset(reset_id) => format!("reset #{reset_id}"),
        }
    }

    fn cell_name(&self, cell_id: CellId) -> String {
        for node in &self.nodes {
            match node {
                Node::Simple { .. } if cell_id == node.target_cell_id() => return format!("Cell ID {}", node.path()),
                Node::Reg { set_cell_id, .. } if cell_id == *set_cell_id => return format!("Cell ID {}$set", node.path()),
                Node::Reg { val_cell_id, .. } if cell_id == *val_cell_id => return format!("Cell ID {}$val", node.path()),
                _ => (),
            }
        }
        panic!()
    }

    fn eval(&self, comb: &Comb) -> Value {
        // TODO associate values with free variables
        let mut ctx: Context<Path, Value> = Context::empty();
        for reference in comb.expr.references() {
            let full_path: Path = comb.rel.join(&reference);
            let cell_id = self.get_node(&full_path).read_cell_id();
            let value = self.get_cell(cell_id).clone();
            ctx = ctx.extend(reference, value);
        }
        eval(ctx, &comb.expr)
    }

    fn get_node(&self, path: &Path) -> &Node {
        for node in &self.nodes {
            if path == node.path() {
                return &node;
            }
        }
        panic!()
    }

    fn get_cell(&self, cell_id: CellId) -> &Value {
        &self.cells[cell_id]
    }

    fn get_cell_mut(&mut self, cell_id: CellId) -> &mut Value {
        &mut self.cells[cell_id]
    }

    pub fn poke(&mut self, path: Path, value: Value) {
        let node = self.get_node(&path);
        let cell_id = node.target_cell_id();
        let cell = self.get_cell_mut(cell_id);
        *cell = value;

        self.events.push(Event::CellUpdated(cell_id));
        self.flow();
    }

    pub fn clock(&mut self) {
        let clock_id = 0; // TODO
        self.events.push(Event::Clock(clock_id));
        self.flow();
    }

    pub fn reset(&mut self) {
        let reset_id = 0; // TODO
        self.events.push(Event::Reset(reset_id));
        self.flow();
    }
}

impl std::fmt::Display for Sim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for node in &self.nodes {
            match node {
                Node::Simple { cell_id, .. } => {
                    write!(f, "{} : {} = ", node.path(), node.type_of())?;
                    writeln!(f, "{}", *self.get_cell(*cell_id))?;
                },
                Node::Reg { set_cell_id, val_cell_id, .. } => {
                    write!(f, "{} : {} = ", node.path(), node.type_of())?;
                    writeln!(f, "{} <= {}", *self.get_cell(*val_cell_id), *self.get_cell(*set_cell_id))?;
                },
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum Node {
//    Clockgen {
//        clock_id: ClockId,
//    },
    Simple {
        path: Path,
        typ: Type,
        cell_id: CellId,
        update: Comb,
    },
    Reg {
        path: Path,
        typ: Type,
        val_cell_id: CellId,
        set_cell_id: CellId,
        update: Comb,
        reset: Option<Value>,
    },
}

impl Node {
    pub fn type_of(&self) -> Type {
        match self {
            Node::Simple { typ, .. } => typ.clone(),
            Node::Reg { typ, .. } => typ.clone(),
        }
    }

    pub fn read_cell_id(&self) -> CellId {
        match self {
            Node::Simple { cell_id, .. } => *cell_id,
            Node::Reg { val_cell_id, .. } => *val_cell_id,
        }
    }

    pub fn target_cell_id(&self) -> CellId {
        match self {
            Node::Simple { cell_id, .. } => *cell_id,
            Node::Reg { set_cell_id, .. } => *set_cell_id,
        }
    }

    pub fn update(&self) -> &Comb {
        match self {
            Node::Simple { update, .. } => update,
            Node::Reg { update, .. } => update,
        }
    }

    fn update_mut(&mut self) -> &mut Comb {
        match self {
            Node::Simple { update, .. } => update,
            Node::Reg { update, .. } => update,
        }
    }

    fn path(&self) -> &Path {
        match self {
            Node::Simple { path, .. } => path,
            Node::Reg { path, .. } => path,
        }
    }
}

#[derive(Debug, Clone)]
struct Comb {
    rel: Path,
    expr: Expr,
    sensitivities: Vec<CellId>,
}

impl Comb {
    fn is_sensitive_to(&self, cell_id: CellId) -> bool {
        self.sensitivities.contains(&cell_id)
    }

    fn is_constant(&self) -> bool {
        self.sensitivities.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Event {
    CellUpdated(CellId),
    Clock(ClockId),
    Reset(ResetId),
}
