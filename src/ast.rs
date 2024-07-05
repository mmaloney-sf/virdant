use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    ModDef(ModDef),
    StructTypeDef(StructTypeDef),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModDef {
    pub name: Ident,
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTypeDef {
    pub name: Ident,
    pub fields: Vec<(Ident, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    SimpleComponent(SimpleComponent),
    Submodule(Submodule),
    Wire(Wire),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleComponent {
    pub name: Ident,
    pub kind: SimpleComponentKind,
    pub typ: Type,
    pub clock: Option<Path>,
    pub reset: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Clock,
    Word(Width),
    Vec(Box<Type>, usize),
    TypeRef(Ident),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleComponentKind {
    Incoming,
    Outgoing,
    Wire,
    Reg,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Reference(Path),
    Word(WordLit),
    Vec(Vec<Expr>),
    Struct(Ident, Vec<(Field, Box<Expr>)>),
    MethodCall(Box<Expr>, Ident, Vec<Expr>),
    As(Box<Expr>, Type),
    Idx(Box<Expr>, StaticIndex),
    IdxRange(Box<Expr>, StaticIndex, StaticIndex),
    Cat(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire(pub Path, pub WireType, pub Expr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum WireType {
    Continuous,
    Latched,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Submodule {
    pub name: Ident,
    pub moddef: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordLit {
    pub value: u64,
    pub width: Option<Width>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WithEdit {
    Idx(u64, Box<Expr>),
    Field(Field, Box<Expr>),
}
