//! Utilities for parsing a Virdant source file.
//!
//! [`parse_package()`](parse_package) is used to parse a package.
//! This results in a [`Ast`] object (or a [`ParseError`] on failure).

use pest::error::Error;
use pest::error::LineColLocation;
use pest::iterators::Pair;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Parser;

/// A node of the parse tree
#[derive(Debug, Clone)]
pub struct Ast<'a>(Pair<'a, Rule>);

/// Parse a Virdant package
pub fn parse_package(text: &str) -> Result<Ast, ParseError> {
    use pest::Parser as PestParser;
    Parser::parse(Rule::package, text)
        .map(|mut pairs| Ast(pairs.next().unwrap()))
        .map_err(|err| ParseError(err))
}

/// A line-col pair (1-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos(usize, usize);

/// A start-end position pair
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(Pos, Pos);

impl<'a> Ast<'a> {
    /// What rule produced this node in the parse tree?
    fn rule(&self) -> Rule {
        self.pair().as_rule()
    }

    /// Get a child node with a given tag.
    pub fn get(&'a self, tag: &'a str) -> Option<Ast<'a>> {
        self.pair().clone().into_inner().find_first_tagged(tag).map(|pair| Ast(pair))
    }

    /// Get the underlying string for a child node with a given tag.
    pub fn get_as_str(&'a self, tag: &'a str) -> Option<&'a str> {
        self.pair().clone().into_inner().find_first_tagged(tag).map(|pair| pair.as_str())
    }

    /// Get the span in the source file for this node of the parse tree.
    pub fn span(&self) -> Span {
        let span = self.pair().as_span();
        let (start_line, start_col) = span.start_pos().line_col();
        let (end_line, end_col) = span.end_pos().line_col();
        Span(Pos(start_line, start_col), Pos(end_line, end_col))
    }

    /// Get the child nodes of this node in the parse tree.
    pub fn children(&self) -> impl Iterator<Item = Ast> {
        let inner = self.pair().clone().into_inner();
        inner
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| Ast(pair))
    }

    pub fn child(&self, i: usize) -> Ast {
        let inner = self.pair().clone().into_inner();
        inner
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| Ast(pair))
            .nth(i)
            .unwrap()
    }

    /// Get the underlying string for this node of the parse tree.
    pub fn as_str(&self) -> &str {
        self.pair().as_str()
    }

    fn pair(&self) -> &Pair<'_, Rule> {
        &self.0
    }

    pub fn is_item(&self) -> bool { self.rule() == Rule::item }
    pub fn is_import(&self) -> bool { self.rule() == Rule::import }

    pub fn package(&self) -> Option<&str> { self.get_as_str("package") }
    pub fn name(&self) -> Option<&str> { self.get_as_str("name") }
    pub fn typ(&self) -> Option<&str> { self.get_as_str("type") }
    pub fn of(&self) -> Option<&str> { self.get_as_str("of") }
    pub fn expr(&self) -> Option<&str> { self.get_as_str("expr") }
}

impl Pos {
    /// The line numbrer (1-indexed).
    pub fn line(&self) -> usize {
        self.0
    }

    /// The column numbrer (1-indexed).
    pub fn col(&self) -> usize {
        self.1
    }
}

impl Span {
    /// The start position.
    pub fn start(&self) -> Pos {
        self.0
    }

    /// The end position.
    pub fn end(&self) -> Pos {
        self.1
    }
}

/// An error encountered during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(Error<Rule>);

impl ParseError {
    /// Where in the source file did the span occur.
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

    pub fn message(&self) -> String {
        "Syntax Error".to_owned()
    }

    fn err(&self) -> &Error<Rule> {
        &self.0
    }
}