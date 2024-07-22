use crate::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AstId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub imports: Vec<PackageImport>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageImport(pub AstId, pub Ident);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    ModDef(ModDef),
    StructDef(StructDef),
    UnionDef(UnionDef),
    PortDef(PortDef),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModDef {
    pub ast_id: AstId,
    pub name: Ident,
    pub decls: Vec<Decl>,
    pub ext: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub ast_id: AstId,
    pub name: Ident,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field(pub Ident, pub Arc<Type>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef {
    pub ast_id: AstId,
    pub name: Ident,
    pub alts: Vec<Alt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alt(pub Ident, pub Vec<Arc<Type>>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortDef {
    pub ast_id: AstId,
    pub name: Ident,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelDir {
    Mosi,
    Miso,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Channel(pub ChannelDir, pub Ident, pub Arc<Type>);


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Port(Port),
    Wire(Wire),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub name: Ident,
    pub kind: ComponentKind,
    pub typ: Arc<Type>,
    pub clock: Option<Arc<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Clock,
    Word(Width),
    Vec(Arc<Type>, usize),
    TypeRef(QualIdent),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentKind {
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
    Struct(Option<QualIdent>, Vec<(Ident, Arc<Expr>)>),
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
    pub moddef: QualIdent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortRole {
    Master,
    Slave,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    pub name: Ident,
    pub role: PortRole,
    pub portdef: QualIdent,
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
