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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(String);

impl<S> From<S> for Path where S: ToString {
    fn from(s: S) -> Path {
        Path(s.to_string())
    }
}

impl Path {
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
