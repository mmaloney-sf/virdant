//use std::collections::HashMap;
use super::*;

type CellId = usize;
type ClockId = usize;

#[derive(Debug, Clone)]
pub struct Sim {
    nodes: Vec<Node>,
    cells: Vec<Value>,
    events: Vec<Event>,
}

impl Sim {
    pub fn new() -> Sim {
        let mut sim = Sim {
            nodes: vec![
                Node::Simple {
                    path: "top.out".into(),
                    typ: Type::Word(8),
                    cell_id: 0,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Reference(Type::Word(8), "a".into()),
                        sensitivities: vec![1],
                    },
                },
                Node::Simple {
                    path: "top.a".into(),
                    typ: Type::Word(8),
                    cell_id: 1,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Word(8, 4),
                        sensitivities: vec![],
                    },
                },
                Node::Reg {
                    path: "top.r".into(),
                    typ: Type::Word(8),
                    val_cell_id: 2,
                    set_cell_id: 3,
                    reset: None,
                    update: Comb {
                        rel: "top".into(),
                        expr: typeinfer(
                            Context::from(vec![
                                ("r".into(), Type::Word(8)),
                                ("a".into(), Type::Word(8))
                            ]),
                            &parse_expr("r->add(a)").unwrap()
                        ),
                        sensitivities: vec![1, 2],
                    },
                    clock: Comb {
                        rel: "top".into(),
                        expr: typeinfer(
                            Context::from(vec![("r".into(), Type::Word(8))]),
                            &parse_expr("r").unwrap()
                        ),
                        sensitivities: vec![1],
                    },
                },
            ].into_iter().collect(),
            cells: vec![
                Value::Word(8, 0),
                Value::Word(8, 0),
                Value::Word(8, 0),
                Value::Word(8, 0),
            ],
            events: vec![Event::Initialize],
        };
        sim.flow();
        sim
    }

    pub fn add_simple_node(&mut self, path: Path, typ: Type, expr: TypedExpr) {
        let cell_id = self.cells.len();

        let sensitivities: Vec<CellId> = expr
            .references()
            .iter()
            .map(|path| todo!())
            .collect();

        let update = Comb {
            rel: path.parent(),
            expr,
            sensitivities,
        };

        let node = Node::Simple {
            cell_id,
            path: path.clone(),
            typ: typ.clone(),
            update,
        };

        self.nodes.push(node);
        self.cells.push(Value::X(typ.clone()));

    }

    fn flow(&mut self) {
        while let Some(event) = self.events.pop() {
            for node in &self.nodes.clone() {
                match (event, node) {
                    (Event::Initialize, Node::Simple { cell_id, typ, update, .. }) => {
                        if update.is_constant() {
                            let value = self.eval(update);
                            self.cells[*cell_id] = value;
                            self.events.push(Event::CellUpdated(*cell_id));
                        } else {
                            self.cells[*cell_id] = Value::X(typ.clone());
                        }
                    },
                    (Event::Initialize, Node::Reg { set_cell_id, val_cell_id, update, typ, .. }) => {
                        if update.is_constant() {
                            let value = self.eval(update);
                            self.cells[*set_cell_id] = value;
                            self.events.push(Event::CellUpdated(*set_cell_id));
                        } else {
                            self.cells[*set_cell_id] = Value::X(typ.clone());
                        }
                        self.cells[*val_cell_id] = Value::X(typ.clone());
                    },
                    (Event::CellUpdated(updated_cell_id), Node::Simple { cell_id, update, .. }) => {
                        if update.is_sensitive_to(updated_cell_id) {
                            let value = self.eval(update);
                            self.cells[*cell_id] = value;
                            self.events.push(Event::CellUpdated(*cell_id));
                        }
                    },
                    (Event::CellUpdated(updated_cell_id), Node::Reg { set_cell_id, update, .. }) => {
                        if update.is_sensitive_to(updated_cell_id) {
                            let value = self.eval(update);
                            self.cells[*set_cell_id] = value;
                            self.events.push(Event::CellUpdated(*set_cell_id));
                        }
                    },
                    (Event::Clock, Node::Reg { set_cell_id, val_cell_id, .. }) => {
                        self.cells[*val_cell_id] = self.cells[*set_cell_id].clone();
                        self.events.push(Event::CellUpdated(*val_cell_id));
                    },
                    _ => (),

                }
            }
        }
    }

    fn event_name(&self, event: &Event) -> String {
        match event {
            Event::Initialize => "initialize".to_string(),
            Event::CellUpdated(cell_id) => format!("updated cell: {}", self.cell_name(*cell_id)),
            Event::Clock => "clock".to_string(),
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
        self.events.push(Event::Clock);
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
        clock: Comb,
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
    expr: TypedExpr,
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
    Initialize,
    CellUpdated(CellId),
    Clock,
}
