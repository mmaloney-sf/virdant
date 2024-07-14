mod astq;
mod item_resolution;
mod item_dependency;
mod item_structure;
mod type_resolution;

use crate::common::*;

#[salsa::database(
    astq::AstQStorage,
    item_resolution::ItemResolutionQStorage,
    item_dependency::ItemDependencyQStorage,
    item_structure::ItemStructureQStorage,
    type_resolution::TypeResolutionQStorage,
)]
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Package(Package),
    ModDef(ModDef),
    UnionDef(UnionDef),
    StructDef(StructDef),
    PortDef(PortDef),
}

impl From<Item> for Path {
    fn from(item: Item) -> Self {
        match item {
            Item::Package(package) => package.into(),
            Item::ModDef(moddef) => moddef.into(),
            Item::UnionDef(uniondef) => uniondef.into(),
            Item::StructDef(structdef) => structdef.into(),
            Item::PortDef(portdef) => portdef.into(),
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

define_fq_type!(Package);

// Items
define_fq_type!(ModDef);
define_fq_type!(UnionDef);
define_fq_type!(StructDef);
define_fq_type!(PortDef);

define_fq_type!(Component);
define_fq_type!(Alt);
define_fq_type!(Field);

pub trait AsItem {
    fn as_item(&self) -> Item;
}

impl AsItem for ModDef {
    fn as_item(&self) -> Item {
        Item::ModDef(self.clone())
    }
}

impl AsItem for UnionDef {
    fn as_item(&self) -> Item {
        Item::UnionDef(self.clone())
    }
}

impl AsItem for StructDef {
    fn as_item(&self) -> Item {
        Item::StructDef(self.clone())
    }
}

impl AsItem for PortDef {
    fn as_item(&self) -> Item {
        Item::PortDef(self.clone())
    }
}

impl Component {
    pub fn moddef(&self) -> ModDef {
        let path: Path = self.clone().into();
        ModDef(path.parent())
    }
}

pub trait FQName {
    fn fqname(&self) -> Path;

    fn package(&self) -> Package {
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
    Union(UnionDef, Vec<TypeArg>),
    Struct(StructDef, Vec<TypeArg>),
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
    use self::astq::*;
    use self::item_resolution::*;
    use self::item_dependency::*;
    use self::item_structure::*;
    use self::type_resolution::*;

    let mut db = Db::default();
    let sources: Vec<(String, Arc<String>)> = vec![
        (
            "edge".to_string(), 
            Arc::new("
                mod EdgeDetector {
                    incoming clock : Clock;
                    incoming inp : Word[1];
                    incoming out : Word[1];

                    reg prev_inp : Word[1] on clock;
                    prev_inp <= inp;

                    out := inp->and(prev_inp->not());
                }
            ".to_string()),
        ),
        (
            "test".to_string(), 
            Arc::new("
                import edge;

                union type ValidByte {
                    Invalid();
                    Valid(Word[8]);
                }

                union type Foo {
                    Foo();
                    Bar(Word[1]);
                }

                union type Bar {
                }

                mod Test {
                    incoming inp : Foo;
                    incoming inp2 : Bar;

                    mod submod of Submod;
                    mod edge of edge.EdgeDetector;
                }

                mod Submod {
                }
            ".to_string()),
        ),
    ];
    db.set_sources(sources.into_iter().collect());
    let package = db.package_ast(Path::from("test").into()).unwrap();
    eprintln!("package:");
    eprintln!("{package:?}");
    eprintln!();

    let items = db.items(Path::from("test").into()).unwrap();

    eprintln!("ITEMS: {items:?}");
    eprintln!();

    let deps = db.moddef_item_dependencies(ModDef("test.Test".into())).unwrap();
    eprintln!("DEPS:");
    eprintln!("{deps:?}");
    eprintln!();


    let components = db.moddef_components(ModDef("test.Test".into())).unwrap();
    eprintln!("test.Test COMPONENTS:");
    eprintln!("{components:?}");
    eprintln!();

    let alts = db.uniondef_alts(UnionDef("test.ValidByte".into())).unwrap();
    eprintln!("test.ValidByte ALTS");
    eprintln!("{alts:?}");
    eprintln!();

    let typ = db.typ("Word".into(), vec![TypeArg::Nat(8)], package.clone()).unwrap();
    eprintln!("Type of Word[8] is:");
    eprintln!("{typ:?}");
    eprintln!();
}
