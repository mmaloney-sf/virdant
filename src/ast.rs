use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

pub type Ident = String;
pub type Width = u64;
pub type UnOp = String;
pub type BinOp = String;
pub type Field = String;

#[derive(Debug, Clone)]
pub enum Expr {
    Reference(Path),
    Word(WordLit),
    Bool(bool),
    Vec(Vec<Expr>),
    Struct(String, Vec<(Field, Box<Expr>)>),
//    If(Box<Expr>, Box<Expr>, Box<Expr>),
//    Match(Box<Expr>, Vec<MatchArm>),
//    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
    FnCall(Ident, Vec<Expr>),
    MethodCall(Box<Expr>, Ident, Vec<Expr>),
    Cat(Vec<Expr>),
    IdxField(Box<Expr>, Ident),
    Idx(Box<Expr>, u64),
    IdxRange(Box<Expr>, u64, u64),
    With(Box<Expr>, Vec<WithEdit>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(String);

impl<S> From<S> for Path where S: ToString {
    fn from(s: S) -> Path {
        Path(s.to_string())
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
