pub use crate::expr::Expr;
use crate::types::Type;
use crate::common::*;

#[derive(Debug, Clone)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    ModDef(ModDef),
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Connect(Connect),
}

#[derive(Debug, Clone)]
pub enum Component {
    Incoming(Ident, Type),
    Outgoing(Ident, Type, Option<Expr>),
    Wire(Ident, Type, Option<Expr>),
    Reg(Ident, Type, Expr, Option<Expr>, Option<Expr>), // Reg(name, clk, rst, set)
}

#[derive(Debug, Clone)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

#[derive(Debug, Clone, Copy)]
pub enum ConnectType {
    Continuous,
    Latched,
}

#[derive(Debug, Clone)]
pub struct Submodule(pub Ident, pub Ident);

#[derive(Debug, Clone)]
pub struct WordLit {
    pub value: u64,
    pub width: Option<Width>,
}

#[derive(Debug, Clone)]
pub enum WithEdit {
    Idx(u64, Box<Expr>),
    Field(Field, Box<Expr>),
}
