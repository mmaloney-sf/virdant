use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::{Expr, Package};

pub fn parse_expr(expr_text: &str) -> Result<Expr, ParseError<usize, Token<'_>, &'static str>> {
    grammar::ExprParser::new().parse(expr_text).map(|expr| *expr)
}

pub fn parse_package(package_text: &str) -> Result<Package, ParseError<usize, Token<'_>, &'static str>> {
    grammar::PackageParser::new().parse(package_text)
}
