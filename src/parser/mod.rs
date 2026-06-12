pub mod ast;
pub mod error;
pub mod lexer;
// parser::parser inception is tolerated until the Phase 1.5 parser consolidation.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod tokens;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lexer_tests;

pub use ast::{Expression, ExpressionKind, MatchPattern, ObjectItem, Program, Span, Statement};
pub use error::ParseError;
pub use lexer::Lexer;
pub use parser::{ParseResult, Parser};
pub use tokens::{Token, TokenType};

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
