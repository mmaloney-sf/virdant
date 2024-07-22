use std::marker::PhantomData;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::Package;
use crate::common::*;
use crate::phase::PackageId;

struct IdGen(PackageId, usize);

impl IdGen {
    fn new(package: &str) -> Self {
        IdGen(PackageId::from_ident(package.into()), 0)
    }

    pub(crate) fn id<T>(&mut self) -> Id<T> {
        let ast_id = Id(self.1, PhantomData::default());
        self.1 += 1;
        ast_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<T>(usize, PhantomData<T>);

impl<T: Clone> Copy for Id<T> {}

pub trait HasId where Self: Sized {
    fn id(&self) -> Id<Self>;
}

pub fn parse_package(package_text: &str) -> Result<Package, VirdantError> {
    let package_name = "top"; // TODO
    let mut gen = IdGen::new(package_name);
    let result: Result<Package, ParseError<usize, Token<'_>, &'static str>>
        = grammar::PackageParser::new().parse(&mut gen, package_text);
    match result {
        Ok(package) => Ok(package),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}
