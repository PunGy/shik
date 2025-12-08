use crate::parser::tokens::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedChar {
        char: char,
        line: usize,
        column: usize,
    },

    InvalidNumber {
        lexeme: String,
        line: usize,
        column: usize,
    },

    ReservedIdentifier {
        lexeme: String,
        line: usize,
        column: usize,
    },

    UnterminatedString {
        line: usize,
        column: usize,
    },

    UnterminatedInterpolationString {
        line: usize,
        column: usize,
    },

    UnexpectedToken {
        token: Token,
        expected: String,
    },

    UnexpectedEndOfInput {
        expected: String,
    },

    InvalidPattern {
        message: String,
        line: usize,
        column: usize,
    },
}

impl ParseError {
    pub fn unexpected_char(ch: char, line: usize, column: usize) -> Self {
        Self::UnexpectedChar {
            char: ch,
            line,
            column,
        }
    }

    pub fn invalid_number(lexeme: String, line: usize, column: usize) -> Self {
        Self::InvalidNumber {
            lexeme,
            line,
            column,
        }
    }

    pub fn reserved_identifier(lexeme: String, line: usize, column: usize) -> Self {
        Self::ReservedIdentifier {
            lexeme,
            line,
            column,
        }
    }

    pub fn unexpected_token(token: Token, expected: String) -> Self {
        Self::UnexpectedToken { token, expected }
    }

    pub fn unexpected_end_of_input(expected: String) -> Self {
        Self::UnexpectedEndOfInput { expected }
    }

    pub fn invalid_pattern(message: String, line: usize, column: usize) -> Self {
        Self::InvalidPattern {
            message,
            line,
            column,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedChar { char, line, column } => {
                write!(
                    f,
                    "Unexpected character '{}' at line {}, column {}",
                    char, line, column
                )
            }
            ParseError::InvalidNumber {
                lexeme,
                line,
                column,
            } => {
                write!(
                    f,
                    "Invalid number '{}' at line {}, column {}",
                    lexeme, line, column
                )
            }
            ParseError::ReservedIdentifier {
                lexeme,
                line,
                column,
            } => {
                write!(
                    f,
                    "Reserved identifier '{}' at line {}, column {}",
                    lexeme, line, column
                )
            }
            ParseError::UnterminatedString { line, column } => {
                write!(f, "Unterminated string at line {}, column {}", line, column)
            }
            ParseError::UnterminatedInterpolationString { line, column } => {
                write!(
                    f,
                    "Unterminated interpolation string at line {}, column {}",
                    line, column
                )
            }
            ParseError::UnexpectedToken { token, expected } => {
                write!(
                    f,
                    "Unexpected token '{}' at line {}, column {}. Expected: {}",
                    token.lexeme, token.line, token.column, expected
                )
            }
            ParseError::UnexpectedEndOfInput { expected } => {
                write!(f, "Unexpected end of input. Expected: {}", expected)
            }
            ParseError::InvalidPattern {
                message,
                line,
                column,
            } => {
                write!(
                    f,
                    "Invalid pattern: {} at line {}, column {}",
                    message, line, column
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
