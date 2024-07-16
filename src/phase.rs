pub mod astq;
pub mod imports;
pub mod item_resolution;
pub mod item_namespace;
pub mod item_dependency;
pub mod type_resolution;
pub mod structure;
pub mod typecheck;
pub mod check;
pub mod layout;

use crate::common::*;

use std::collections::HashMap;

#[salsa::database(
    astq::AstQStorage,
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
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

impl Db {
    pub fn new() -> Db {
        use self::astq::*;
        let mut db = Db::default();
        let sources = HashMap::new();
        db.set_sources(sources);
        db
    }

    pub fn set_source(&mut self, package: &str, text: &str) {
        use self::astq::*;
        let mut sources = self.sources();
        sources.insert(package.into(), Arc::new(text.to_string()));
        self.set_sources(sources);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
    ModDef(ModDefId),
    UnionDef(UnionDefId),
    StructDef(StructDefId),
    PortDef(PortDefId),
}

impl From<ItemId> for Path {
    fn from(item: ItemId) -> Self {
        match item {
            ItemId::ModDef(moddef) => moddef.into(),
            ItemId::UnionDef(uniondef) => uniondef.into(),
            ItemId::StructDef(structdef) => structdef.into(),
            ItemId::PortDef(portdef) => portdef.into(),
        }
    }
}

impl FQName for ItemId {
    fn fqname(&self) -> Path {
        match self {
            ItemId::ModDef(moddef) => moddef.fqname(),
            ItemId::UnionDef(uniondef) => uniondef.fqname(),
            ItemId::StructDef(structdef) => structdef.fqname(),
            ItemId::PortDef(portdef) => portdef.fqname(),
        }
    }
}

macro_rules! define_fq_type {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct $name(Path);

        impl From<$name> for Path {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FQName for $name {
            fn fqname(&self) -> Path {
                self.0.clone().into()
            }
        }

        impl From<Path> for $name {
            fn from(value: Path) -> Self {
                $name(value)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $name {
            pub fn name(&self) -> Ident {
                let parts = self.0.parts();

                parts[parts.len() - 1].clone().into()
            }
        }
    };
}

define_fq_type!(PackageId);

// Items
define_fq_type!(ModDefId);
define_fq_type!(UnionDefId);
define_fq_type!(StructDefId);
define_fq_type!(PortDefId);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementId {
    Component(ComponentId),
    Alt(AltId),
    Field(FieldId),
}

impl FQName for ElementId {
    fn fqname(&self) -> Path {
        match self {
            ElementId::Component(component) => component.fqname(),
            ElementId::Alt(alt) => alt.fqname(),
            ElementId::Field(field) => field.fqname(),
        }
    }
}

define_fq_type!(ComponentId);
define_fq_type!(AltId);
define_fq_type!(FieldId);

pub trait AsElement {
    fn as_element(&self) -> ElementId;
}

impl AsElement for ComponentId {
    fn as_element(&self) -> ElementId {
        ElementId::Component(self.clone())
    }
}

impl AsElement for AltId {
    fn as_element(&self) -> ElementId {
        ElementId::Alt(self.clone())
    }
}

impl AsElement for FieldId {
    fn as_element(&self) -> ElementId {
        ElementId::Field(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MethodSig(Vec<Type>, Type);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct CtorSig(Vec<Type>, Type);

pub trait AsItem {
    fn as_item(&self) -> ItemId;
}

impl AsItem for ModDefId {
    fn as_item(&self) -> ItemId {
        ItemId::ModDef(self.clone())
    }
}

impl AsItem for UnionDefId {
    fn as_item(&self) -> ItemId {
        ItemId::UnionDef(self.clone())
    }
}

impl AsItem for StructDefId {
    fn as_item(&self) -> ItemId {
        ItemId::StructDef(self.clone())
    }
}

impl AsItem for PortDefId {
    fn as_item(&self) -> ItemId {
        ItemId::PortDef(self.clone())
    }
}

impl ComponentId {
    pub fn moddef(&self) -> ModDefId {
        let path: Path = self.clone().into();
        ModDefId(path.parent())
    }
}

pub trait FQName {
    fn fqname(&self) -> Path;

    fn name(&self) -> Ident {
        let fqname = &self.fqname();
        let parts = fqname.parts();
        Path::from(parts[parts.len() - 1]).as_ident().unwrap()
    }

    fn package(&self) -> PackageId {
        let fqname = &self.fqname();
        let parts = fqname.parts();
        Path::from(parts[0]).into()
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

    /*
    let package_ast = db.package_ast(Path::from("test").into()).unwrap();
    eprintln!("package:");
    eprintln!("{package_ast:?}");
    eprintln!();

    let items = db.items(Path::from("test").into()).unwrap();

    eprintln!("ITEMS: {items:?}");
    eprintln!();

    let deps = db.moddef_item_dependencies(ModDefId("test.Test".into())).unwrap();
    eprintln!("DEPS:");
    eprintln!("{deps:?}");
    eprintln!();
*/
}

pub fn compile_verilog(input: &str) -> VirdantResult<()> {
    let mut db = Db::new();
    db.set_source("top", input);

    let mut stdout = std::io::stdout();
    db.verilog(&mut stdout)?;
    Ok(())
}
