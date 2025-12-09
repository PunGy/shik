#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    StringInterpolation(StringInterpolationInfo),
    Ident,

    // Keywords
    Let,
    Fn,
    Pipe, // $>
    Flow, // #>
    Chain, // $

    // Delimiters
    LeftParen,         // (
    RightParen,        // )
    LeftBracket,       // [
    RightBracket,      // ]
    LeftCurlyBracket,  // {
    RightCurlyBracket, // }

    // Comment
    BlockComment,
    SingleLineComment, // ;

    OpenBlock, // '(
    OpenLazy,  // #(

    // Special
    Hash,    // meta keyword
    Newline, // new line
    Eof,
}

/**
 * For interpolated string, the following original string: "hello {. user :name}, what is the {random-question}"
 * the interpolation would be:

StringInterpolationInfo {
    string: "hello _, what is the _"
    entries: [
        Interpolation { tokens: ["." "user" ":name"], start: 6, end: 19, position: 6 },
        Interpolation { tokens: ["random-question"], start: 34, end: 50, position: 21 },
    ]
}
 */
#[derive(Debug, Clone, PartialEq)]
pub struct StringInterpolationInfo {
    // Interpolated string, with all interpolations replaced with placeholder "_"
    pub string: String,
    pub entries: Vec<Interpolation>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Interpolation {
    pub tokens: Vec<Token>,
    pub start: usize,
    pub end: usize,

    // position of interpolation in the wrapped string
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }

    pub fn ident(lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Ident,
            lexeme,
            line,
            column,
        }
    }

    pub fn eof(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line,
            column,
        }
    }
    pub fn hash(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Hash,
            lexeme: "#".to_string(),
            line,
            column,
        }
    }

    pub fn left_paren(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::LeftParen,
            lexeme: "(".to_string(),
            line,
            column,
        }
    }
    pub fn right_paren(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::RightParen,
            lexeme: ")".to_string(),
            line,
            column,
        }
    }
    pub fn left_bracket(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::LeftBracket,
            lexeme: "[".to_string(),
            line,
            column,
        }
    }
    pub fn right_bracket(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::RightBracket,
            lexeme: "]".to_string(),
            line,
            column,
        }
    }
    pub fn left_curly_bracket(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::LeftCurlyBracket,
            lexeme: "{".to_string(),
            line,
            column,
        }
    }
    pub fn right_curly_bracket(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::RightCurlyBracket,
            lexeme: "}".to_string(),
            line,
            column,
        }
    }

    pub fn open_block(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::OpenBlock,
            lexeme: "'(".to_string(),
            line,
            column,
        }
    }
    pub fn open_lazy(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::OpenLazy,
            lexeme: "#(".to_string(),
            line,
            column,
        }
    }

    pub fn flow(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Flow,
            lexeme: "#>".to_string(),
            line,
            column,
        }
    }

    pub fn block_comment(lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::BlockComment,
            lexeme,
            line,
            column,
        }
    }
    pub fn single_line_comment(lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::SingleLineComment,
            lexeme,
            line,
            column,
        }
    }

    pub fn newline(line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Newline,
            lexeme: "\n".to_string(),
            line,
            column,
        }
    }
}
