use std::collections::HashSet;
use crate::*;
use crate::ast::WithEdit;
use crate::common::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Reference(Type, Path),
    Word(Option<Width>, u64),
    Bool(bool),
    Vec(Type, Vec<Expr>),
    Struct(Type, Vec<(Field, Box<Expr>)>),
//    If(Box<Expr>, Box<Expr>, Box<Expr>),
//    Match(Box<Expr>, Vec<MatchArm>),
//    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
    FnCall(Ident, Vec<Expr>),
    MethodCall(Type, Box<Expr>, Ident, Vec<Expr>),
    Cat(Vec<Expr>),
    IdxField(Box<Expr>, Ident),
    Idx(Box<Expr>, u64),
    IdxRange(Box<Expr>, u64, u64),
    With(Box<Expr>, Vec<WithEdit>),
}

impl Expr {
    pub fn type_of(&self) -> Type {
        match self {
            Expr::Reference(typ, _path) => typ.clone(),
            Expr::Word(width, _value) => Type::Word(width.unwrap()),
            Expr::Bool(_b) => Type::Bool,
            Expr::Vec(typ, es) => Type::Vec(Box::new(typ.clone()), es.len()),
            Expr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    Expr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    Expr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    Expr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
//            Expr::Call(Ident, Vec<Expr>) => todo!(),
//            Expr::Cat(Vec<Expr>) => todo!(),
//            Expr::IdxField(Box<Expr>, Ident) => todo!(),
//            Expr::Idx(Box<Expr>, u64) => todo!(),
//            Expr::IdxRange(Box<Expr>, u64, u64) => todo!(),
//            Expr::With(Box<Expr>, Vec<WithEdit>) => todo!(),
            _ => todo!()
        }
    }

    pub fn references(&self) -> HashSet<Path> {
        match self {
            Expr::Reference(_typ, path) => vec![path.clone()].into_iter().collect(),
            Expr::Word(_width, _value) => HashSet::new(),
            Expr::Bool(_b) => HashSet::new(),
            Expr::Vec(_typ, es) => {
                let mut result = HashSet::new();
                for e in es {
                    result.extend(e.references());
                }
                result
            },
            Expr::MethodCall(_typ, subject, _fun, args) => {
                let mut result = HashSet::new();
                result.extend(subject.references());
                for e in args {
                    result.extend(e.references());
                }
                result
            },
            //Expr::Struct(typ, _flds) => Type::Other(typ.to_string()),
        //    Expr::If(Box<Expr>, Box<Expr>, Box<Expr>) => todo!(),
        //    Expr::Match(Box<Expr>, Vec<MatchArm>) => todo!(),
        //    Expr::Let(Ident, Option<Type>, Box<Expr>, Box<Expr>) => todo!(),
//            Expr::Cat(Vec<Expr>) => todo!(),
//            Expr::IdxField(Box<Expr>, Ident) => todo!(),
//            Expr::Idx(Box<Expr>, u64) => todo!(),
//            Expr::IdxRange(Box<Expr>, u64, u64) => todo!(),
//            Expr::With(Box<Expr>, Vec<WithEdit>) => todo!(),
            _ => todo!()
        }
    }
}


