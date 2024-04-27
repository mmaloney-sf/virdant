use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;
use crate::expr::Expr;

pub type Ident = String;
pub type Width = u64;
pub type UnOp = String;
pub type BinOp = String;
pub type Field = String;

#[derive(Debug, Clone)]
pub struct Package(Vec<Def>);

#[derive(Debug, Clone)]
pub enum Def {
    ModDef(ModDef),
}

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Ident,
    pub components: Vec<Component>,
    pub connect: Vec<Connect>,
    pub submodules: Vec<Submodule>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Component(Component),
    Submodule(Submodule),
    Connect(Connect),
}

#[derive(Debug, Clone)]
pub enum Component {
    Input(Ident),
    Output(Ident, Option<Expr>),
    Node(Ident, Option<Expr>),
    Reg(Ident, Expr, Option<Expr>, Option<Expr>), // Reg(name, clk, rst, set)
}

#[derive(Debug, Clone)]
pub struct Connect(Path, ConnectType, Expr);

#[derive(Debug, Clone)]
pub enum ConnectType {
    Direct,
    Latched,
    Signal,
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
