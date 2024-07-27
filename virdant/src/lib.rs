/// Utilities for parsing a Virdant source file.
///
/// [`parse_package()`](parse::parse_package) is used to parse a package.
/// This results in a [`ParseTree`](parse::ParseTree) object (or a [`ParseError`](parse::ParseError) on failure).
pub mod parse;

#[cfg(test)]
mod tests;

