use pest::error::Error;
use pest::error::LineColLocation;
use pest::iterators::Pair;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Parser;

#[derive(Debug, Clone)]
pub struct ParseTree<'a>(Pair<'a, Rule>);

pub fn parse_package(text: &str) -> Result<ParseTree, ParseError> {
    use pest::Parser as PestParser;
    let r = Parser::parse(Rule::package, text)
        .map(|mut pairs| ParseTree(pairs.next().unwrap()))
        .map_err(|err| ParseError(err));

    dbg!(&r);
    r
}

#[derive(Debug, Clone, Copy)]
pub struct Pos(usize, usize);

#[derive(Debug, Clone, Copy)]
pub struct Span(Pos, Pos);

impl<'a> ParseTree<'a> {
    pub fn rule(&self) -> Rule {
        self.pair().as_rule()
    }

    pub fn get(&'a self, tag: &'a str) -> Option<ParseTree<'a>> {
        self.pair().clone().into_inner().find_first_tagged(tag).map(|pair| ParseTree(pair))
    }

    pub fn get_as_str(&'a self, tag: &'a str) -> Option<&'a str> {
        self.pair().clone().into_inner().find_first_tagged(tag).map(|pair| pair.as_str())
    }

    pub fn span(&self) -> Span {
        let span = self.pair().as_span();
        let (start_line, start_col) = span.start_pos().line_col();
        let (end_line, end_col) = span.end_pos().line_col();
        Span(Pos(start_line, start_col), Pos(end_line, end_col))
    }

    pub fn children(&self) -> impl Iterator<Item = ParseTree> {
        let inner = self.pair().clone().into_inner();
        inner
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| ParseTree(pair))
    }

    pub fn child(&self, i: usize) -> ParseTree {
        let inner = self.pair().clone().into_inner();
        inner
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| ParseTree(pair))
            .nth(i)
            .unwrap()
    }

    pub fn as_str(&self) -> &str {
        self.pair().as_str()
    }

    fn pair(&self) -> &Pair<'_, Rule> {
        &self.0
    }
}

impl Pos {
    pub fn line(&self) -> usize {
        self.0
    }

    pub fn col(&self) -> usize {
        self.1
    }
}

impl Span {
    pub fn start(&self) -> Pos {
        self.0
    }

    pub fn end(&self) -> Pos {
        self.1
    }
}

#[derive(Debug, Clone)]
pub struct ParseError(Error<Rule>);

impl ParseError {
    pub fn span(&self) -> Span {
        match self.err().line_col {
            LineColLocation::Pos((line, col)) => {
                let start = Pos(line, col);
                let end = Pos(line, col);
                Span(start, end)
            },
            LineColLocation::Span(start, end) => {
                let (start_line, start_col) = start;
                let (end_line, end_col) = end;
                Span(Pos(start_line, start_col), Pos(end_line, end_col))
            },
        }

    }

    fn err(&self) -> &Error<Rule> {
        &self.0
    }
}
