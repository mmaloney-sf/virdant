mod astq;
mod item_resolution;

use crate::common::*;

#[salsa::database(
    astq::AstQStorage,
    item_resolution::ItemResolutionQStorage,
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

macro_rules! define_path_type {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct $name(Path);

        impl From<$name> for Path {
            fn from(value: $name) -> Self {
                value.0
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
    };
}

define_path_type!(Package);
define_path_type!(ModDef);
define_path_type!(UnionDef);
define_path_type!(StructDef);


#[test]
fn phase() {
    use self::item_resolution::*;
    use self::astq::*;

    let mut db = Db::default();
    db.set_package_source("test".into(), Arc::new("
        mod Test {
        }

        alt type Foo {
        }
    ".into()));
    let package = db.package_ast(Path::from("test").into()).unwrap();
    eprintln!("package:");
    eprintln!("{package:?}");

    let items = db.items(Path::from("test").into()).unwrap();

    eprintln!("ITEMS: {items:?}");
}
