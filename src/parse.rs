use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::{Expr, Package};
use crate::common::*;



pub fn parse_expr(expr_text: &str) -> Result<Expr, VirdantError> {
    let result: Result<Expr, ParseError<usize, Token<'_>, &'static str>> = grammar::ExprParser::new().parse(expr_text).map(|expr| *expr);
    match result {
        Ok(expr) => Ok(expr),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}

pub fn parse_package(package_text: &str) -> Result<Package, VirdantError> {
    let result: Result<Package, ParseError<usize, Token<'_>, &'static str>> = grammar::PackageParser::new().parse(package_text);
    match result {
        Ok(package) => Ok(package),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}
