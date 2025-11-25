use crate::parser::tokens::{Interpolation, StringInterpolationInfo, Token, TokenType};

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
}
impl ParseError {
    fn unexpected_char(ch: char, line: usize, column: usize) -> Self {
        Self::UnexpectedChar {
            char: ch,
            line,
            column,
        }
    }

    fn invalid_number(lexeme: String, line: usize, column: usize) -> Self {
        Self::InvalidNumber {
            lexeme,
            line,
            column,
        }
    }
    fn reserved_identifier(lexeme: String, line: usize, column: usize) -> Self {
        Self::ReservedIdentifier {
            lexeme,
            line,
            column,
        }
    }
}

pub type TokenizeResult = ParseResult<Vec<Token>>;

fn is_digit(ch: Option<char>) -> bool {
    ch.map_or(false, |ch| ch.is_ascii_digit())
}

const IDENT_START_CHARSET: &str = "!@%^&*-=_+|?<>.$";
const IDENT_CHARSET: &str = "!@%^&*-=_+|?<>$'.*#";

const INLINE_STRING_SEPARATOR: &str = "\n\r {}()[]";

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_numeric() || IDENT_START_CHARSET.contains(ch)
}
fn is_ident(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_numeric() || IDENT_CHARSET.contains(ch)
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> TokenizeResult {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> ParseResult<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token::eof(self.line, self.column));
        }

        let start_column = self.column;
        let line = self.line;
        let ch = self.advance();

        let token = match ch {
            '(' => Token::left_paren(line, start_column),
            ')' => Token::right_paren(line, start_column),
            '[' => Token::left_bracket(line, start_column),
            ']' => Token::right_bracket(line, start_column),
            '{' => {
                if self.peek() == Some('*') {
                    self.advance(); // skip *
                    return self.comment_block(start_column);
                } else {
                    Token::left_curly_bracket(line, start_column)
                }
            }
            ';' => return self.comment_line(start_column),
            '}' => Token::right_curly_bracket(line, start_column),

            '"' => return self.string_block(start_column),
            ':' => return self.string_inline(start_column),

            '#' => {
                if self.peek_next() == Some('(') {
                    Token::open_lazy(line, start_column)
                } else {
                    Token::hash(line, start_column)
                }
            }
            '\'' => {
                if self.peek() == Some('(') {
                    Token::open_block(line, start_column)
                } else {
                    return Err(ParseError::UnexpectedChar {
                        char: ch,
                        line,
                        column: start_column,
                    });
                }
            }

            '0'..='9' => return self.number(start_column),

            c if is_ident_start(c) => return self.ident(start_column),

            _ => {
                return Err(ParseError::UnexpectedChar {
                    char: ch,
                    line,
                    column: start_column,
                })
            }
        };

        Ok(token)
    }

    fn number(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;

        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == Some('.') && is_digit(self.peek_next()) {
            self.advance(); // skip .

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        match lexeme.parse::<f64>() {
            Ok(value) => Ok(Token {
                token_type: TokenType::Number(value),
                lexeme,
                line: self.line,
                column: start_column,
            }),
            Err(_) => Err(ParseError::invalid_number(lexeme, self.line, start_column)),
        }
    }

    /// Block string - "example", "hello {. user :name}!"
    fn string_block(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;

        let mut interpolation: Option<StringInterpolationInfo> = None;
        loop {
            let ch = self.peek();

            match ch {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    self.advance(); // skip next character
                }
                Some('{') => {
                    let mut depth = 0;
                    // start of interpolation relative to input
                    let i_start = self.current;
                    // start of interpolation relative to string
                    let interpolation_start = self.current - (start + 1);

                    self.advance();
                    loop {
                        let inner = self.peek();
                        match inner {
                            Some('{') => {
                                depth += 1;
                            }
                            Some('}') => {
                                if depth == 0 {
                                    break;
                                } else {
                                    depth -= 1;
                                }
                            }
                            s if s == None || (s == Some('"') && depth == 0) => {
                                return Err(ParseError::UnterminatedInterpolationString {
                                    line: self.line,
                                    column: start_column,
                                })
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }

                    let input: String = self.input[i_start + 1..self.current].iter().collect();
                    let mut inter_lexer = Lexer::new(input.as_str());

                    if interpolation.is_none() {
                        interpolation = Some(StringInterpolationInfo {
                            string: "".to_string(),
                            entries: vec![],
                        })
                    }

                    match inter_lexer.tokenize() {
                        Ok(tokens) => {
                            interpolation.as_mut().unwrap().entries.push(Interpolation {
                                tokens: tokens,
                                start: interpolation_start,
                                end: self.current - (start + 1),

                                position: 0, // to be adjusted further
                            })
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }

                    self.advance();
                }
                None => {
                    return Err(ParseError::UnterminatedString {
                        line: self.line,
                        column: self.column,
                    })
                }
                _ => {
                    self.advance();
                }
            };
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        let token = match interpolation {
            None => Token::new(
                TokenType::String(lexeme[1..lexeme.len() - 1].to_string()),
                lexeme,
                self.line,
                start_column,
            ),
            Some(mut i) => {
                let original = &lexeme[1..lexeme.len() - 1];
                let mut interpolated: String = String::with_capacity(lexeme.len());

                let mut position: usize = 0;
                let mut from: usize = 0;
                for inter in &mut i.entries {
                    interpolated.push_str(&original[from..inter.start]);
                    interpolated.push('_');

                    position += inter.start - from;
                    inter.position = position;

                    from = inter.end + 1;
                }
                interpolated.push_str(&original[from..]);

                i.string = interpolated;

                Token::new(
                    TokenType::StringInterpolation(i),
                    lexeme,
                    self.line,
                    start_column,
                )
            }
        };

        Ok(token)
    }
    /// Inline string - :example
    fn string_inline(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;

        loop {
            let ch = self.peek();

            match ch {
                Some(s) => {
                    if INLINE_STRING_SEPARATOR.contains(s) {
                        break;
                    } else {
                        self.advance();
                    }
                }
                None => {
                    break; // inline string is always terminating
                }
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        Ok(Token::new(
            TokenType::String(lexeme[1..].to_string()),
            lexeme,
            self.line,
            start_column,
        ))
    }

    fn comment_block(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 2;

        let mut depth = 0;
        while let Some(ch) = self.peek() {
            match ch {
                s if s == '{' && self.peek_next() == Some('*') => {
                    self.advance(); // skip {
                    self.advance(); // skip *
                    depth += 1;
                }
                s if s == '*' && self.peek_next() == Some('}') => {
                    self.advance(); // skip *
                    self.advance(); // skip }
                    if depth == 0 {
                        break;
                    } else {
                        depth -= 1;
                    }
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                _ => {
                    self.advance();
                }
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        Ok(Token::block_comment(lexeme, self.line, start_column))
    }
    fn comment_line(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;

        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        let lexeme: String = self.input[start..self.current].iter().collect();

        return Ok(Token::single_line_comment(lexeme, self.line, start_column));
    }

    fn ident(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;

        while let Some(ch) = self.peek() {
            if is_ident(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        let token = match lexeme.as_str() {
            "$>" => Token::new(TokenType::Pipe, lexeme, self.line, start_column),
            "let" => Token::new(TokenType::Let, lexeme, self.line, start_column),
            "fn" => Token::new(TokenType::Fn, lexeme, self.line, start_column),
            _ => Token::ident(lexeme, self.line, start_column),
        };
        Ok(token)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.current).copied()
    }
    fn peek_next(&self) -> Option<char> {
        self.input.get(self.current + 1).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.input[self.current];
        self.column += 1;
        self.current += 1;
        ch
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }
}
