use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    ModDef(ModDef),
    PortDef(PortDef),
    StructTypeDef(StructTypeDef),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
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
pub struct PortDef {
    pub name: Ident,
    pub fields: Vec<PortField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum PortDir {
    Miso,
    Mosi,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum PortKind {
    Master,
    Slave,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortField {
    pub dir: PortDir,
    pub name: Ident,
    pub typ: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTypeDef {
    pub name: Ident,
    pub fields: Vec<(Ident, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Port(Port),
    Connect(Connect),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub name: Ident,
    pub kind: ComponentKind,
    pub typ: Type,
    pub connect: Option<InlineConnect>,
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
pub enum ComponentKind {
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
pub struct Connect(pub Path, pub ConnectType, pub Expr);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ConnectType {
    Continuous,
    Latched,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Submodule {
    pub name: Ident,
    pub moddef: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    pub kind: PortKind,
    pub name: Ident,
    pub portname: Ident,
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
