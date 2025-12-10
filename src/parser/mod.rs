pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod tokens;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lexer_tests;

pub use ast::{Expression, LetPattern, MatchPattern, ObjectItem, Program, Statement};
pub use error::ParseError;
pub use lexer::Lexer;
pub use parser::{Parser, ParseResult};
pub use tokens::{Token, TokenType};

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
