use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    ModDef(ModDef),
    StructTypeDef(StructTypeDef),
    AltTypeDef(AltTypeDef),
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
    pub fields: Vec<(Ident, Arc<Type>)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AltTypeDef {
    pub name: Ident,
    pub alts: Vec<(Ident, Vec<Arc<Type>>)>,
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
    pub typ: Arc<Type>,
    pub clock: Option<Path>,
    pub reset: Option<Arc<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Clock,
    Word(Width),
    Vec(Arc<Type>, usize),
    TypeRef(Ident),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleComponentKind {
    Incoming,
    Outgoing,
    Node,
    Reg,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Reference(Path),
    Word(WordLit),
    Vec(Vec<Arc<Expr>>),
    Struct(Ident, Vec<(Field, Arc<Expr>)>),
    MethodCall(Arc<Expr>, Ident, Vec<Arc<Expr>>),
    Ctor(Ident, Vec<Arc<Expr>>),
    As(Arc<Expr>, Arc<Type>),
    Idx(Arc<Expr>, StaticIndex),
    IdxRange(Arc<Expr>, StaticIndex, StaticIndex),
    Cat(Vec<Arc<Expr>>),
    If(Arc<Expr>, Arc<Expr>, Arc<Expr>),
    Let(Ident, Option<Arc<Type>>, Arc<Expr>, Arc<Expr>),
    Match(Arc<Expr>, Option<Arc<Type>>, Vec<MatchArm>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArm(pub Pat, pub Arc<Expr>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pat {
    At(Ident, Vec<Pat>),
    Bind(Ident),
    Otherwise,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire(pub Path, pub WireType, pub Arc<Expr>);

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
    Idx(u64, Arc<Expr>),
    Field(Field, Arc<Expr>),
}
