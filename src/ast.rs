use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;
use crate::expr::Expr;
use crate::types::Type;

pub type Ident = String;
pub type Width = u64;
pub type UnOp = String;
pub type BinOp = String;
pub type Field = String;

#[derive(Debug, Clone)]
pub struct Package(Vec<Item>);

#[derive(Debug, Clone)]
pub enum Item {
    ModDef(ModDef),
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub connects: Vec<Connect>,
    pub submodules: Vec<Submodule>,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Connect(Connect),
}

#[derive(Debug, Clone)]
pub enum Component {
    Incoming(Ident, Type, ),
    Outgoing(Ident, Type, Option<Expr>),
    Wire(Ident, Type, Option<Expr>),
    Reg(Ident, Type, Expr, Option<Expr>, Option<Expr>), // Reg(name, clk, rst, set)
}

#[derive(Debug, Clone)]
pub struct Connect(Path, ConnectType, Expr);

#[derive(Debug, Clone, Copy)]
pub enum ConnectType {
    Continuous,
    Latched,
}

#[derive(Debug, Clone)]
pub struct Submodule(Ident, Ident);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(String);

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<S> From<S> for Path where S: Into<String> {
    fn from(s: S) -> Path {
        Path(s.into())
    }
}

impl Path {
    pub fn join(&self, other: &Path) -> Path {
        format!("{}.{}", self.0, other.0).into()
    }

    pub fn parent(&self) -> Path {
        let parts = self.parts();
        parts[0..parts.len()-1].join(".").into()
    }

    pub fn parts(&self) -> Vec<&str> {
        self.0.split('.').collect()
    }

    pub fn is_local(&self) -> bool {
        self.parts().len() == 1
    }

    pub fn is_foreign(&self) -> bool {
        self.parts().len() == 2
    }

    pub fn is_remote(&self) -> bool {
        self.parts().len() > 2
    }
}

#[test]
fn path_tests() {
    let p1: Path = "top.foo".into();
    let p2: Path = "top.foo.bar".into();
    assert_eq!(p2.parent(), p1);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordLit(pub Option<Width>, pub u64);

impl WordLit {
    pub fn width(&self) -> Option<Width> {
        self.0
    }

    pub fn val(&self) -> u64 {
        self.1
    }
}

#[derive(Debug, Clone)]
pub enum WithEdit {
    Idx(u64, Box<Expr>),
    Field(Field, Box<Expr>),
}

pub fn parse_expr(expr_text: &str) -> Result<Expr, ParseError<usize, Token<'_>, &'static str>> {
    grammar::ExprParser::new().parse(expr_text).map(|expr| *expr)
}

pub fn parse_package(package_text: &str) -> Result<Package, ParseError<usize, Token<'_>, &'static str>> {
    grammar::PackageParser::new().parse(package_text)
}

impl Package {
    pub fn moddefs(&self) -> Vec<&ModDef> {
        let mut result = Vec::new();
        for def in &self.0 {
            match def {
                Item::ModDef(moddef) => result.push(moddef),
            }
        }
        result
    }
}

pub enum NamedItem<'a> {
    Component(&'a Component),
    Submodule(&'a Submodule),
}

impl Connect {
    pub fn target(&self) -> Path {
        self.0.clone()
    }

    pub fn connect_type(&self) -> ConnectType {
        self.1
    }

    pub fn expr(&self) -> Expr {
        self.2.clone()
    }
}

impl ModDef {
    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn components(&self) -> &[Component] {
        &self.components
    }

    pub fn submodules(&self) -> &[Submodule] {
        &self.submodules
    }

    /// given a name, get either the component or submodule with that name, or None if none exists
    /// (panic if there is more than one)
    pub fn get(&self, name: &str) -> Option<&NamedItem> {
        todo!()
    }

    /// returns a ist containing all paths which may appear as reference in any expression in the moddef
    /// that is, all wires, all incoming ports, all registers, and the outgoing ports of every submodule
    /// cf the `tap` expression, which can refer to a target path
    pub fn visible_reference_paths(&self) -> Vec<Path> {
        todo!()
    }

    /// returns a ist containing all paths which may appear as a target for any connect in the moddef
    /// that is, all wires, all outgoing ports, all registers, and the incoming ports of every submodule
    pub fn visible_target_paths(&self) -> Vec<Path> {
        todo!()
    }

    /// given the name of a component, return the connect for it.
    /// In the case that a component is declared and connected to in the same declaration,
    /// return a synthetic Connect instead.
    pub fn connect_for(&self, component_name: &str) -> Connect {
        todo!()
        /*
        for connect in &self.connects {
        }

        panic!()
        */
    }
}

impl Package {
    pub fn items(&self) -> &[Item] {
        &self.0
    }
}

impl Component {
    pub fn name(&self) -> &Ident {
        match self {
            Component::Incoming(name, _typ) => &name,
            Component::Outgoing(name, _typ, e) => &name,
            Component::Wire(name, _typ, e) => &name,
            Component::Reg(name, _typ, clock, reset, e) => &name,
        }
    }

    pub fn type_of(&self) -> &Type {
        match self {
            Component::Incoming(name, typ) => &typ,
            Component::Outgoing(name, typ, e) => &typ,
            Component::Wire(name, typ, e) => &typ,
            Component::Reg(name, typ, clock, reset, e) => &typ,
        }
    }
}

impl Submodule {
    pub fn name(&self) -> &Ident {
        &self.0
    }

    pub fn moddef(&self) -> &Ident {
        &self.1
    }
}
