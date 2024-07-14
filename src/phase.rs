mod astq;
mod item_resolution;
mod item_dependency;

use crate::{common::*, phase::item_dependency::ItemDependencyQ};

#[salsa::database(
    astq::AstQStorage,
    item_resolution::ItemResolutionQStorage,
    item_dependency::ItemDependencyQStorage,
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

macro_rules! define_path_type {
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

define_path_type!(Package);
define_path_type!(ModDef);
define_path_type!(UnionDef);
define_path_type!(StructDef);

pub trait FQName {
    fn fqname(&self) -> Path;
}

impl ModDef {
    pub fn package(&self) -> Package {
        self.0.parent().into()
    }
}

#[test]
fn phase() {
    use self::item_resolution::*;
    use self::astq::*;

    let mut db = Db::default();
    let sources: Vec<(String, Arc<String>)> = vec![
        (
            "test".to_string(), 
            Arc::new("
                alt type Foo {
                }

                alt type Bar {
                }

                mod Test {
                    incoming inp : Foo;
                    incoming inp2 : Bar;

                    mod submod of Submod;
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
}
