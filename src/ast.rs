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

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Connect(Connect),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: Ident,
    pub kind: ComponentKind,
    pub typ: Type,
    pub connect: Option<InlineConnect>,
    pub clock: Option<Expr>,
    pub reset: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Incoming,
    Outgoing,
    Wire,
    Reg,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Reference(Path),
    Word(WordLit),
    Vec(Vec<Expr>),
    Struct(Ident, Vec<(Field, Box<Expr>)>),
    MethodCall(Box<Expr>, Ident, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

#[derive(Debug, Clone)]
pub struct InlineConnect(pub ConnectType, pub Expr);

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
