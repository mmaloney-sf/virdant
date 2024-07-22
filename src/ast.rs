use std::marker::PhantomData;
use crate::common::*;
use crate::loc::{Span, SourceInfo};
use crate::phase::id::PackageId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ast<T>(Arc<T>, Span, Id<T>);

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

pub struct AstGen {
    package_id: PackageId, 
    source_info: SourceInfo, 
    next_id: usize,
}

impl AstGen {
    pub fn new(package: &str) -> Self {
        let package_id = PackageId::from_ident(package.into()); 
        let source_info: SourceInfo = SourceInfo::unknown();
        let next_id = 0;
        AstGen {
            package_id,
            source_info,
            next_id,
        }
    }

    fn id<T>(&mut self) -> Id<T> {
        let ast_id = Id(self.next_id, PhantomData::default());
        self.next_id += 1;
        ast_id
    }

    pub fn ast<T>(&mut self, t: T, span_start_idx: usize, span_end_idx: usize) -> Ast<T> {
        let id = self.id();
        let span: Span = Span::from(&self.source_info, span_start_idx, span_end_idx);
        Ast(Arc::new(t), span, id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<T>(usize, PhantomData<T>);

impl<T: Clone> Copy for Id<T> {}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef {
    pub name: Ident,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field(pub Ident, pub Arc<Type>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef {
    pub name: Ident,
    pub alts: Vec<Alt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alt(pub Ident, pub Vec<Arc<Type>>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortDef {
    pub name: Ident,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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

impl<T: Clone> Ast<T> {
    pub fn id(&self) -> Id<T> {
        self.2.clone()
    }
}
