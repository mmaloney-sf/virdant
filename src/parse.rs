use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

use crate::ast::{Package, AstGen, Ast};
use crate::common::*;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PackageParser;

#[test]
fn parse() {
    use pest::Parser;
    let example = "

import foo;

mod Foo {
    incoming clock : Clock;
    outgoing out : Word[8];
    port mem of Mem;

    reg counter : Word[8] on clock;
    modfoo of Foo;

    foo.inp := counter;
}
";
    let parse = dbg!(PackageParser::parse(Rule::package, example)).unwrap();
    for token in parse.tokens() {
        eprintln!("{token:?}");
    }
}

pub fn parse_package(package_name: &str, package_text: &str) -> VirdantResult<Ast<Package>> {
    let mut gen = AstGen::new(package_name);
    let result: Result<Ast<Package>, ParseError<usize, Token<'_>, &'static str>>
        = grammar::PackageParser::new().parse(&mut gen, package_text);
    match result {
        Ok(package) => Ok(package),
        Err(err) => Err(VirdantError::ParseError(format!("{err:?}"))),
    }
}
