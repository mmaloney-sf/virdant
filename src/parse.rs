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

mod lr {
    #![allow(dead_code)]

    #[derive(PartialEq, Eq, Clone, Debug, Hash)]
    enum Token {
        C,
        D,
    }

    #[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
    enum Nt {
        Start,
        S,
        C,
    }

    enum Slot {
        Token(Token),
        Nt(Nt),
        End,
    }

    type ProdId = usize;

    struct Prod(Nt, Vec<Slot>);

    struct Grammar {
        prods: Vec<Prod>,
    }

    struct Item {
        prod: Prod,
        dot: usize,
    }

    use std::collections::HashSet;

    impl Grammar {
        fn production(&self, production_id: ProdId) -> &Prod {
            &self.prods[production_id]
        }

        fn productions_by_nt(&self, nt: Nt) -> Vec<&Prod> {
            let mut result = vec![];
            for production@Prod(lhs, rhs) in &self.prods {
                if *lhs == nt {
                    result.push(production);
                }
            }
            result
        }

        fn first(&self, nt: Nt) -> HashSet<Token> {
            let mut result = HashSet::new();
            for Prod(lhs, rhs) in &self.prods {
                if *lhs == nt {
                    let first_slot = rhs.first().unwrap();
                    match first_slot {
                        Slot::Token(c) => {
                            result.insert(c.clone());
                        },
                        Slot::Nt(nt2) => {
                            result.extend(self.first(*nt2).iter().cloned());
                        },
                        Slot::End => (),
                    }
                }
            }
            result
        }
    }

    #[test]
    fn lr_test() {
        let grammar = Grammar {
            prods: vec![
                Prod(Nt::Start, vec![Slot::Nt(Nt::S), Slot::End]),
                Prod(Nt::S, vec![Slot::Nt(Nt::C), Slot::Nt(Nt::C)]),
                Prod(Nt::C, vec![Slot::Token(Token::C), Slot::Nt(Nt::C)]),
                Prod(Nt::C, vec![Slot::Token(Token::D)]),
            ],
        };

        dbg!(grammar.first(Nt::Start));
        assert!(false);
    }
}
