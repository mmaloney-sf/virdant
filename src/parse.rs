use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::AstId;
use crate::ast::Package;
use crate::common::*;

struct AstGen(Ident, AstId);

impl AstGen {
    fn new(package: &str) -> Self {
        AstGen(package.into(), AstId(0))
    }

    pub(crate) fn id(&mut self) -> AstId {
        let ast_id = self.1.clone();
        self.1.0 += 1;
        ast_id
    }
}


pub fn parse_package(package_text: &str) -> Result<Package, VirdantError> {
    let package_name = "top"; // TODO
    let mut gen = AstGen::new(package_name);
    let result: Result<Package, ParseError<usize, Token<'_>, &'static str>>
        = grammar::PackageParser::new().parse(&mut gen, package_text);
    match result {
        Ok(package) => Ok(package),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}
