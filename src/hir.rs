mod expr;
mod passes;

use std::sync::Arc;
use crate::common::*;
use crate::ast;
use crate::types::Type;
use crate::ast::ConnectType;

pub use expr::Expr;
pub use expr::ExprNode;
pub use expr::IsExpr;

#[derive(Debug, Clone)]
pub struct Package {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    ModDef(ModDef),
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub submodules: Vec<Submodule>,
    pub connects: Vec<Connect>,
}

#[derive(Debug, Clone)]
pub struct Submodule {
    pub name: Ident,
    pub moddef_name: Ident,
}

#[derive(Debug, Clone)]
pub enum Component {
    Incoming(Ident, Arc<Type>),
    Outgoing(Ident, Arc<Type>, Option<InlineConnect>),
    Wire(Ident, Arc<Type>, Option<InlineConnect>),
    Reg(Ident, Arc<Type>, Option<Expr>, /*Option<Value>,*/ Option<InlineConnect>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HirType {
    Clock,
    Word(Width),
    Vec(Arc<Type>, usize),
}

#[derive(Debug, Clone)]
pub struct InlineConnect(pub ConnectType, pub Expr);

#[derive(Debug, Clone)]
pub struct Connect(pub Path, pub ConnectType, pub Expr);

impl InlineConnect {
    pub fn from_ast(connect: &ast::InlineConnect) -> InlineConnect {
        let ast::InlineConnect(connect_type, expr) = connect;
        InlineConnect(*connect_type, Expr::from_ast(expr))
    }
}

impl Package {
    pub fn compile(package: &ast::Package) -> Result<Package, VirdantError> {
        let package = Package::from_ast(package);
        package.run_passes()
    }

    pub fn from_ast(package: &ast::Package) -> Package {
        let mut items = vec![];
        for item in &package.items {
            items.push(Item::from_ast(item))
        }
        Package {
            items,
        }
    }

    pub fn item(&self, name: Ident) -> Option<&Item> {
        for item in &self.items {
            if item.name() == name {
                return Some(item);
            }
        }
        None
    }

    pub fn moddefs(&self) -> Vec<ModDef> {
        let mut moddefs = vec![];
        for item in &self.items {
            match item {
                Item::ModDef(moddef) => moddefs.push(moddef.clone()),
            }
        }
        moddefs
    }
}

impl Item {
    pub fn from_ast(item: &ast::Item) -> Item {
        match item {
            ast::Item::ModDef(moddef) => Item::ModDef(ModDef::from_ast(moddef)),
        }
    }

    fn name(&self) -> Ident {
        match self {
            Item::ModDef(m) => m.name.clone(),
        }
    }
}

impl Component {
    pub fn name(&self) -> Ident {
        match self {
            Component::Incoming(name, _typ) => name.clone(),
            Component::Outgoing(name, _typ, _connect) => name.clone(),
            Component::Wire(name, _typ, _connect) => name.clone(),
            Component::Reg(name, _typ, _clk, /*Option<Value>,*/ _connect) => name.clone(),
        }
    }

    pub fn type_of(&self) -> Arc<Type> {
        match self {
            Component::Incoming(_name, typ) => typ.clone(),
            Component::Outgoing(_name, typ, _connect) => typ.clone(),
            Component::Wire(_name, typ, _connect) => typ.clone(),
            Component::Reg(_name, typ, _clk, /*Option<Value>,*/ _connect) => typ.clone(),
        }
    }

    pub fn connect_mut(&mut self) -> Option<&mut InlineConnect> {
        match self {
            Component::Incoming(_name, _typ) => None,
            Component::Outgoing(_name, _typ, connect) => connect.as_mut(),
            Component::Wire(_name, _typ, connect) => connect.as_mut(),
            Component::Reg(_name, _typ, _clk, /*Option<Value>,*/ connect) => connect.as_mut(),
        }
    }
}

impl Type {
    pub fn from_ast(typ: &ast::Type) -> Arc<Type> {
        match typ {
            ast::Type::Clock => Type::Clock.into(),
            ast::Type::Word(width) => Type::Word(*width).into(),
            ast::Type::Vec(inner, len) => Type::Vec(Type::from_ast(inner), *len).into(),
        }
    }
}

impl ModDef {
    pub fn from_ast(moddef: &ast::ModDef) -> ModDef {
        let name = moddef.name.clone();
        let mut components = vec![];
        let mut submodules = vec![];
        let mut connects = vec![];

        for decl in &moddef.decls {
            match decl {
                ast::Decl::Component(component) => {
                    let connect = component.connect.as_ref().map(|c| InlineConnect::from_ast(&c));
                    let c = match component.kind {
                        ast::ComponentKind::Incoming => Component::Incoming(component.name.clone(), Type::from_ast(&component.typ)),
                        ast::ComponentKind::Outgoing => Component::Outgoing(component.name.clone(), Type::from_ast(&component.typ), connect),
                        ast::ComponentKind::Wire => Component::Wire(component.name.clone(), Type::from_ast(&component.typ), connect),
                        ast::ComponentKind::Reg => Component::Reg(component.name.clone(), Type::from_ast(&component.typ), component.clock.as_ref().map(|e| Expr::from_ast(e)), connect),
                    };
                    components.push(c);
                },
                ast::Decl::Submodule(ast::Submodule(name, moddef_name)) => {
                    submodules.push(
                        Submodule {
                            name: name.clone(),
                            moddef_name: moddef_name.clone(),
                        }
                    );
                },
                ast::Decl::Connect(ast::Connect(path, connect_type, expr)) => {
                    connects.push(Connect(path.clone(), *connect_type, Expr::from_ast(expr)));
                },
            }
        }

        ModDef {
            name,
            components,
            submodules,
            connects,
        }
    }
}
