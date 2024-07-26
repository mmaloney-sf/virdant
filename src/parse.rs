pub use pest::error::Error;
pub use pest::iterators::{Pairs, Pair};

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Parser;

pub fn parse_package(text: &str) -> Result<Pairs<'_, Rule>, Error<Rule>> {
    use pest::Parser as PestParser;
    Parser::parse(Rule::package, text)
}
