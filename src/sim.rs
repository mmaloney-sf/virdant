use std::collections::HashMap;
use super::*;

type CellId = usize;

#[derive(Debug, Clone)]
pub struct Sim {
    nodes: HashMap<Path, Node>,
    cells: Vec<Value>,
    events: Vec<Event>,
}

impl Sim {
    pub fn new() -> Sim {
        let mut sim = Sim {
            nodes: vec![
                ("clock".into(), Node::Clock { cell_id: 0 }),
                ("top.out".into(), Node::Simple {
                    cell_id: 1,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Reference(Type::Word(8), "a".into()),
                        sensitivities: vec![Event::CellUpdated(2)],
                    },
                }),
                ("top.a".into(), Node::Simple {
                    cell_id: 2,
                    update: Comb {
                        rel: "top".into(),
                        expr: TypedExpr::Word(8, 4),
                        sensitivities: vec![Event::Initialize],
                    },
                }),
                ("top.r".into(), Node::Reg {
                    val_cell_id: 3,
                    set_cell_id: 4,
                    reset: None,
                    update: Comb {
                        rel: "top".into(),
                        expr: typeinfer(
                            Context::from(vec![("r".into(), Type::Word(8))]),
                            &parse_expr("r->add(1w8)").unwrap()
                        ),
                        sensitivities: vec![Event::CellUpdated(0)],
                    },
                }),
            ].into_iter().collect(),
            cells: vec![
                Value::Clock(false),
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
            self.flow_event(&event);
        }
    }

    fn flow_event(&mut self, event: &Event) {
        let nodes: Vec<Node> = self.nodes.values().cloned().collect();
        for node in nodes {
            if let Some(comb) = node.update() {
                if comb.sensitivities.contains(event) {
                    let value = self.eval(&comb);
                    let target_cell_id = node.target_cell_id();
                    let cell = self.get_cell_mut(target_cell_id);
                    *cell = value;
                    self.events.push(Event::CellUpdated(target_cell_id));
                }
            }
        }
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
        if let Node::Clock { cell_id } = self.get_node(&"clock".into()) {
            let clock = self.get_cell_mut(*cell_id);
            clock.tick_clock();
            println!("tick");
            clock.tick_clock();
            println!("tock");
        } else {
            panic!();
        }
    }

    fn get_node(&self, path: &Path) -> &Node {
        &self.nodes[path]
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
        cell_id: CellId,
        update: Comb,
    },
    Reg {
        val_cell_id: CellId,
        set_cell_id: CellId,
        update: Comb,
        reset: Option<Value>,
    },
    Clock {
        cell_id: CellId,
    },
}

impl Node {
    pub fn read_cell_id(&self) -> CellId {
        match self {
            Node::Simple { cell_id, .. } => *cell_id,
            Node::Reg { val_cell_id, .. } => *val_cell_id,
            Node::Clock { .. } => panic!(),
        }
    }

    pub fn target_cell_id(&self) -> CellId {
        match self {
            Node::Simple { cell_id, .. } => *cell_id,
            Node::Reg { set_cell_id, .. } => *set_cell_id,
            Node::Clock { .. } => panic!(),
        }
    }

    pub fn update(&self) -> Option<&Comb> {
        match self {
            Node::Simple { update, .. } => Some(update),
            Node::Reg { update, .. } => Some(update),
            Node::Clock { .. } => None,
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
}
