use crate::common::*;
use crate::phase::sourceq::SpanIdx;
use crate::phase::id::PackageId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ast<T>(Arc<T>, SpanIdx, AstId);

impl<T> std::ops::Deref for Ast<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> AsRef<T> for Ast<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> Ast<T> {
    pub fn span(&self) -> SpanIdx {
        self.1.clone()
    }
}

pub struct AstGen {
    package_id: PackageId,
    next_id: usize,
}

impl AstGen {
    pub fn new(package: &str) -> Self {
        let package_id = PackageId::from_ident(package.into());
        let next_id = 0;
        AstGen {
            package_id,
            next_id,
        }
    }

    fn id(&mut self) -> AstId {
        let ast_id = AstId(self.next_id);
        self.next_id += 1;
        ast_id
    }

    pub fn ast<T>(&mut self, t: T, span_start_idx: usize, span_end_idx: usize) -> Ast<T> {
        let id = self.id();
        let span: SpanIdx = SpanIdx::new(self.package_id.clone(), span_start_idx, span_end_idx);
        Ast(Arc::new(t), span, id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct AstId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub imports: Vec<Ast<PackageImport>>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageImport(pub Ident);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    ModDef(Ast<ModDef>),
    StructDef(Ast<StructDef>),
    UnionDef(Ast<UnionDef>),
    PortDef(Ast<PortDef>),
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
    pub ext: bool,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field(pub Ident, pub Ast<Type>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef {
    pub name: Ident,
    pub alts: Vec<Alt>,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alt(pub Ident, pub Vec<Ast<Type>>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortDef {
    pub name: Ident,
    pub channels: Vec<Channel>,
    pub doc: Option<DocComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ChannelDir {
    Mosi,
    Miso,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Channel(pub ChannelDir, pub Ident, pub Ast<Type>);


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    Component(Ast<Component>),
    Submodule(Ast<Submodule>),
    Port(Ast<Port>),
    Wire(Ast<Wire>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub name: Ident,
    pub kind: ComponentKind,
    pub typ: Ast<Type>,
    pub clock: Option<Ast<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Clock,
    Word(Width),
    Vec(Ast<Type>, usize),
    TypeRef(QualIdent),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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
    Vec(Vec<Ast<Expr>>),
    Struct(Option<QualIdent>, Vec<(Ident, Ast<Expr>)>),
    MethodCall(Ast<Expr>, Ident, Vec<Ast<Expr>>),
    Ctor(Ident, Vec<Ast<Expr>>),
    As(Ast<Expr>, Ast<Type>),
    Idx(Ast<Expr>, StaticIndex),
    IdxRange(Ast<Expr>, StaticIndex, StaticIndex),
    Cat(Vec<Ast<Expr>>),
    If(Ast<Expr>, Ast<Expr>, Ast<Expr>),
    Let(Ident, Option<Ast<Type>>, Ast<Expr>, Ast<Expr>),
    Match(Ast<Expr>, Option<Ast<Type>>, Vec<MatchArm>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArm(pub Pat, pub Ast<Expr>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pat {
    At(Ident, Vec<Pat>),
    Bind(Ident),
    Otherwise,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire(pub Path, pub WireType, pub Ast<Expr>);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(pub String);

impl<T: Clone> Ast<T> {
    pub fn id(&self) -> AstId {
        self.2.clone()
    }
}
