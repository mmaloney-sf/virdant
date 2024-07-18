pub mod astq;
pub mod resolve;
pub mod imports;
pub mod item_resolution;
pub mod item_namespace;
pub mod item_dependency;
pub mod type_resolution;
pub mod structure;
pub mod typecheck;
pub mod check;
pub mod layout;

pub mod id;

pub use id::*;

use crate::common::*;

use std::collections::HashMap;

#[salsa::database(
    astq::AstQStorage,
    resolve::ResolveQStorage,
    imports::ImportsQStorage,
    item_resolution::ItemResolutionQStorage,
    item_namespace::ItemNamespaceQStorage,
    item_dependency::ItemDependencyQStorage,
    structure::StructureQStorage,
    type_resolution::TypeResolutionQStorage,
    typecheck::TypecheckQStorage,
    check::CheckQStorage,
    layout::LayoutQStorage,
)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

impl Db {
    pub fn new() -> Db {
        use self::astq::*;
        let mut db = Db {
            storage: salsa::Storage::default(),
        };
        let sources = HashMap::new();
        db.set_sources(sources);
        db
    }

    pub fn set_source(&mut self, package: &str, text: &str) -> PackageId {
        use self::astq::*;
        let mut sources = self.sources();
        sources.insert(package.into(), Arc::new(text.to_string()));
        self.set_sources(sources);
        PackageId::from_ident(package.to_string().into())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeArg {
    Type(Arc<Type>),
    Nat(u64),
}

impl std::fmt::Display for TypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeArg::Type(typ) => write!(f, "{typ}"),
            TypeArg::Nat(n) => write!(f, "{n}"),
        }
    }
}

impl std::fmt::Debug for TypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Clock,
    Bool,
    Word(Width),
    Union(UnionDefId, Vec<TypeArg>),
    Struct(StructDefId, Vec<TypeArg>),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MethodSig(Vec<Type>, Type);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct CtorSig(Vec<Type>, Type);


impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Type::Clock => write!(f, "Clock"),
            Type::Bool => write!(f, "Bool"),
            Type::Word(width) => write!(f, "Word[{width}]"),
            Type::Struct(structdef, args) => {
                write!(f, "{structdef}")?;
                if args.len() > 0 {
                    write!(f, "[")?;
                    for arg in args {
                        write!(f, "{arg}")?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            },
            Type::Union(uniondef, args) => {
                write!(f, "{uniondef}")?;
                if args.len() > 0 {
                    write!(f, "[")?;
                    for arg in args {
                        write!(f, "{arg}")?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            },
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self}")
    }
}

#[test]
fn phase() {
    #![allow(unused)]
    use crate::ast;
    use self::astq::*;
    use self::item_resolution::*;
    use self::item_dependency::*;
    use self::structure::*;
    use self::type_resolution::*;
    use self::check::*;

    let mut db = Db::new();

    let edge_source = "
        mod EdgeDetector {
            incoming clock : Clock;
            incoming inp : Word[1];
            incoming out : Word[1];

            reg prev_inp : Word[1] on clock;
            prev_inp <= inp;

            out := inp->and(prev_inp->not());
        }
    ";

    eprintln!("Setting edge source");
    db.set_source("edge", edge_source);

    let top_source = "
        import edge;

        mod Top {
            mod edge of edge::EdgeDetector;
        }
    ";

    eprintln!("Setting top source");
    db.set_source("top", top_source);

    db.check().unwrap();
}
