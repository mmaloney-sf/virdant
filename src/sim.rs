//use std::collections::HashMap;
use super::*;

type CellId = usize;

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
                    cell_id: 0,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Reference(Type::Word(8), "a".into()),
                        sensitivities: vec![Event::CellUpdated(1)],
                    },
                },
                Node::Simple {
                    path: "top.a".into(),
                    cell_id: 1,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Word(8, 4),
                        sensitivities: vec![Event::Initialize],
                    },
                },
                Node::Reg {
                    path: "top.r".into(),
                    val_cell_id: 2,
                    set_cell_id: 3,
                    reset: None,
                    update: Comb {
                        rel: "top".into(),
                        expr: typeinfer(
                            Context::from(vec![("r".into(), Type::Word(8))]),
                            &parse_expr("r->add(1w8)").unwrap()
                        ),
                        sensitivities: vec![Event::CellUpdated(1)],
                    },
                    clock: Comb {
                        rel: "top".into(),
                        expr: typeinfer(
                            Context::from(vec![("r".into(), Type::Word(8))]),
                            &parse_expr("r").unwrap()
                        ),
                        sensitivities: vec![Event::Clock],
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

    fn flow(&mut self) {
        while let Some(event) = self.events.pop() {
            println!("FLOW: {}", self.event_name(&event));
            self.flow_event(&event);
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
        format!("Cell ID {cell_id}")
    }

    fn nodes_sensitive_to(&self, event: &Event) -> Vec<Node> {
        let mut result = Vec::new();
        for node in &self.nodes {
            if node.sensitive_to(event) {
                result.push(node.clone());
            }
        }
        result
    }

    fn flow_event(&mut self, event: &Event) {
        for node in &self.nodes_sensitive_to(event) {
            self.flow_event_to_node(event, node);
        }
    }

    fn flow_event_to_node(&mut self, event: &Event, node: &Node) {
        let comb = node.update();
        let value = self.eval(&comb);
        let target_cell_id = node.target_cell_id();
        let cell = self.get_cell_mut(target_cell_id);
        *cell = value;
        self.events.push(Event::CellUpdated(target_cell_id));
    }

    fn eval(&self, comb: &Comb) -> Value {
        // TODO associate values with free variables
        let mut ctx: Context<Path, Value> = Context::empty();
        for reference in comb.expr.free_refs() {
            let full_path: Path = comb.rel.join(&reference);
            let cell_id = self.get_node(&full_path).read_cell_id();
            let value = self.get_cell(cell_id).clone();
            ctx = ctx.extend(reference, value);
        }
        eval(ctx, &comb.expr)
    }

    pub fn clock(&mut self) {
        println!("TICK - - - - - - - - - - - - - - - - - - - - - - - - - - - - ");
        self.events.push(Event::Clock);
        self.flow();
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
}

#[derive(Debug, Clone)]
enum Node {
    Simple {
        path: Path,
        cell_id: CellId,
        update: Comb,
    },
    Reg {
        path: Path,
        val_cell_id: CellId,
        set_cell_id: CellId,
        update: Comb,
        clock: Comb,
        reset: Option<Value>,
    },
}

impl Node {
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

    fn sensitive_to(&self, event: &Event) -> bool {
        match self {
            Node::Simple { update, .. } => update.sensitivities.contains(event),
            Node::Reg { update, clock, .. } => update.sensitivities.contains(event) || clock.sensitivities.contains(event),
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
    sensitivities: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Initialize,
    CellUpdated(CellId),
    Clock,
}
