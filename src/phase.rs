pub mod astq;
pub mod item_resolution;
pub mod item_dependency;
pub mod structure;
pub mod type_resolution;
pub mod typecheck;
pub mod layout;

use crate::common::*;

#[salsa::database(
    astq::AstQStorage,
    item_resolution::ItemResolutionQStorage,
    item_dependency::ItemDependencyQStorage,
    structure::StructureQStorage,
    type_resolution::TypeResolutionQStorage,
    typecheck::TypecheckQStorage,
    layout::LayoutQStorage,
)]
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
    Package(PackageId),
    ModDef(ModDefId),
    UnionDef(UnionDefId),
    StructDef(StructDefId),
    PortDef(PortDefId),
}

impl From<ItemId> for Path {
    fn from(item: ItemId) -> Self {
        match item {
            ItemId::Package(package) => package.into(),
            ItemId::ModDef(moddef) => moddef.into(),
            ItemId::UnionDef(uniondef) => uniondef.into(),
            ItemId::StructDef(structdef) => structdef.into(),
            ItemId::PortDef(portdef) => portdef.into(),
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

define_fq_type!(ComponentId);
define_fq_type!(AltId);
define_fq_type!(FieldId);

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
    use crate::ast;
    use self::astq::*;
    use self::item_resolution::*;
    use self::item_dependency::*;
    use self::structure::*;
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


    let components = db.moddef_components(ModDefId("test.Test".into())).unwrap();
    eprintln!("test.Test COMPONENTS:");
    eprintln!("{components:?}");
    eprintln!();

    let alts = db.uniondef_alts(UnionDefId("test.ValidByte".into())).unwrap();
    eprintln!("test.ValidByte ALTS");
    eprintln!("{alts:?}");
    eprintln!();

    eprintln!("Component types:");
    for component in &components {
        eprint!("    {component} : ");
        let moddef_ast = db.moddef_ast(component.moddef()).unwrap();
        let mut has_type = false;
        for decl in &moddef_ast.decls {
            if let ast::Decl::SimpleComponent(simplecomponent) = decl {
                if simplecomponent.name == component.name() {
                    let package = PackageId("test".into());
                    let typ = db.resolve_typ(simplecomponent.typ.clone(), package).unwrap();
                    eprintln!("{typ:?}");
                    has_type = true;
                }
            }
        }
        if !has_type {
            eprintln!("(submodule)");
        }
    }
    eprintln!();
}

pub fn compile_verilog(input: &str) -> VirdantResult<()> {
    use crate::phase::astq::AstQ;

    let mut db = Db::default();
    let sources = vec![("top".to_string(), Arc::new(input.to_string()))];
    db.set_sources(sources.into_iter().collect());

    let mut stdout = std::io::stdout();
    db.verilog(&mut stdout)?;
    Ok(())
}
