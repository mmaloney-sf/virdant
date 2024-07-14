mod astq;
mod item_resolution;
mod item_dependency;
mod item_structure;

use crate::common::*;

#[salsa::database(
    astq::AstQStorage,
    item_resolution::ItemResolutionQStorage,
    item_dependency::ItemDependencyQStorage,
    item_structure::ItemStructureQStorage,
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
}

impl From<Item> for Path {
    fn from(item: Item) -> Self {
        match item {
            Item::Package(package) => package.into(),
            Item::ModDef(moddef) => moddef.into(),
            Item::UnionDef(uniondef) => uniondef.into(),
            Item::StructDef(structdef) => structdef.into(),
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

pub trait FQName {
    fn fqname(&self) -> Path;

    fn package(&self) -> Package {
        let fqname = &self.fqname();
        let parts = fqname.parts();
        Path::from(parts[0]).into()
    }
}

#[test]
fn phase() {
    use self::item_dependency::*;
    use self::item_resolution::*;
    use self::astq::*;
    use self::item_structure::*;

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

                alt type ValidByte {
                    Invalid();
                    Valid(Word[8]);
                }

                alt type Foo {
                    Foo();
                    Bar(Word[1]);
                }

                alt type Bar {
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
}
