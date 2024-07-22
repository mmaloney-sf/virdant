use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::{Package, AstGen, Ast};
use crate::common::*;

pub fn parse_package(package_name: &str, package_text: &str) -> VirdantResult<Ast<Package>> {
    let mut gen = AstGen::new(package_name);
    let result: Result<Ast<Package>, ParseError<usize, Token<'_>, &'static str>>
        = grammar::PackageParser::new().parse(&mut gen, package_text);
    match result {
        Ok(package) => Ok(package),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}
