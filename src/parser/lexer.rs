use crate::parser::error::ParseError;
use crate::parser::tokens::{Interpolation, StringInterpolationInfo, Token, TokenType};

pub type TokenizeResult = ParseResult<Vec<Token>>;

fn is_digit(ch: Option<char>) -> bool {
    ch.map_or(false, |ch| ch.is_ascii_digit())
}

const IDENT_START_CHARSET: &str = "!@%^&*-=_+|?<>.$/";
const IDENT_CHARSET: &str = "!@%^&*-=_+|?<>$'.*#/";

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
            // newline termination
            if self.peek() == Some('\n') {
                tokens.push(Token::newline(0, 0));
            }
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
                if self.peek() == Some('(') {
                    self.advance(); // skip (
                    Token::open_lazy(line, start_column)
                } else if self.peek() == Some('>') {
                    self.advance(); // skip >
                    Token::flow(line, start_column)
                } else {
                    Token::hash(line, start_column)
                }
            }
            '\'' => {
                if self.peek() == Some('(') {
                    self.advance(); // skip (
                    Token::open_block(line, start_column)
                } else {
                    return Err(ParseError::UnexpectedChar {
                        char: ch,
                        line,
                        column: start_column,
                    });
                }
            }

            '-' => {
                if self
                    .peek()
                    .map(|x| ('0'..='9').contains(&x))
                    .unwrap_or(false)
                {
                    self.advance();
                    return self.number(start_column, true);
                } else {
                    return self.ident(start_column);
                }
            }
            '0'..='9' => return self.number(start_column, false),

            '$' => match self.peek() {
                Some('>') => {
                    self.advance();
                    Token::new(TokenType::Pipe, "$>".to_string(), line, start_column)
                }
                Some('\n') | Some(' ') | Some('\t') | Some('\r') => {
                    Token::new(TokenType::Chain, "$".to_string(), line, start_column)
                }
                _ => return self.ident(start_column),
            },

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

    fn number(&mut self, start_column: usize, negative: bool) -> ParseResult<Token> {
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
                token_type: TokenType::Number(if negative { -value } else { value }),
                lexeme,
                line: self.line,
                column: start_column,
            }),
            Err(_) => Err(ParseError::invalid_number(lexeme, self.line, start_column)),
        }
    }

    /// Process an escape sequence starting after the backslash.
    /// Returns the escaped character or an error for invalid sequences.
    fn process_escape_sequence(&mut self, start_column: usize) -> ParseResult<char> {
        let escape_col = self.column;
        let ch = self.peek();

        match ch {
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some('\'') => {
                self.advance();
                Ok('\'')
            }
            Some('0') => {
                self.advance();
                Ok('\0')
            }
            Some('{') => {
                self.advance();
                Ok('{')
            }
            Some('}') => {
                self.advance();
                Ok('}')
            }
            Some('x') => {
                // Hex escape: \xNN
                self.advance(); // consume 'x'
                let hex_start = self.current;
                for _ in 0..2 {
                    match self.peek() {
                        Some(c) if c.is_ascii_hexdigit() => {
                            self.advance();
                        }
                        _ => {
                            let seq: String = self.input[hex_start..self.current].iter().collect();
                            return Err(ParseError::invalid_escape_sequence(
                                format!("x{}", seq),
                                self.line,
                                escape_col,
                            ));
                        }
                    }
                }
                let hex_str: String = self.input[hex_start..self.current].iter().collect();
                let code = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                    ParseError::invalid_escape_sequence(
                        format!("x{}", hex_str),
                        self.line,
                        escape_col,
                    )
                })?;
                Ok(code as char)
            }
            Some('u') => {
                // Unicode escape: \u{NNNN} or \u{NNNNNN}
                self.advance(); // consume 'u'
                if self.peek() != Some('{') {
                    return Err(ParseError::invalid_escape_sequence(
                        "u".to_string(),
                        self.line,
                        escape_col,
                    ));
                }
                self.advance(); // consume '{'
                let hex_start = self.current;
                while let Some(c) = self.peek() {
                    if c == '}' {
                        break;
                    }
                    if c.is_ascii_hexdigit() {
                        self.advance();
                    } else {
                        let seq: String = self.input[hex_start..self.current].iter().collect();
                        return Err(ParseError::invalid_escape_sequence(
                            format!("u{{{}", seq),
                            self.line,
                            escape_col,
                        ));
                    }
                }
                if self.peek() != Some('}') {
                    let seq: String = self.input[hex_start..self.current].iter().collect();
                    return Err(ParseError::invalid_escape_sequence(
                        format!("u{{{}", seq),
                        self.line,
                        escape_col,
                    ));
                }
                let hex_str: String = self.input[hex_start..self.current].iter().collect();
                self.advance(); // consume '}'

                if hex_str.is_empty() || hex_str.len() > 6 {
                    return Err(ParseError::invalid_escape_sequence(
                        format!("u{{{}}}", hex_str),
                        self.line,
                        escape_col,
                    ));
                }

                let code = u32::from_str_radix(&hex_str, 16).map_err(|_| {
                    ParseError::invalid_escape_sequence(
                        format!("u{{{}}}", hex_str),
                        self.line,
                        escape_col,
                    )
                })?;
                char::from_u32(code).ok_or_else(|| {
                    ParseError::invalid_escape_sequence(
                        format!("u{{{}}}", hex_str),
                        self.line,
                        escape_col,
                    )
                })
            }
            Some(c) => Err(ParseError::invalid_escape_sequence(
                c.to_string(),
                self.line,
                escape_col,
            )),
            None => Err(ParseError::UnterminatedString {
                line: self.line,
                column: start_column,
            }),
        }
    }

    /// Block string - "example", "hello {. user :name}!"
    fn string_block(&mut self, start_column: usize) -> ParseResult<Token> {
        let start = self.current - 1;
        let mut content = String::new();
        let mut interpolation: Option<StringInterpolationInfo> = None;

        loop {
            let ch = self.peek();

            match ch {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance(); // consume backslash
                    let escaped = self.process_escape_sequence(start_column)?;
                    content.push(escaped);
                }
                Some('{') => {
                    // start of interpolation relative to processed content
                    let interpolation_start = content.len();

                    self.advance();
                    let i_start = self.current;
                    loop {
                        let inner = self.peek();
                        match inner {
                            Some('}') => {
                                break;
                            }
                            Some('"') => {
                                return Err(ParseError::UnterminatedInterpolationString {
                                    line: self.line,
                                    column: start_column,
                                });
                            }
                            None => {
                                return Err(ParseError::UnterminatedInterpolationString {
                                    line: self.line,
                                    column: start_column,
                                });
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }

                    let input: String = self.input[i_start..self.current].iter().collect();
                    let mut inter_lexer = Lexer::new(input.as_str());

                    if interpolation.is_none() {
                        interpolation = Some(StringInterpolationInfo {
                            string: String::new(),
                            entries: vec![],
                        })
                    }

                    match inter_lexer.tokenize() {
                        Ok(tokens) => {
                            interpolation.as_mut().unwrap().entries.push(Interpolation {
                                tokens,
                                start: interpolation_start,
                                end: interpolation_start + 1, // placeholder position
                                position: interpolation_start,
                            });
                            content.push('_'); // placeholder for interpolation
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }

                    self.advance(); // consume '}'
                }
                None => {
                    return Err(ParseError::UnterminatedString {
                        line: self.line,
                        column: self.column,
                    })
                }
                Some(c) => {
                    content.push(c);
                    self.advance();
                }
            };
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        let token = match interpolation {
            None => Token::new(
                TokenType::String(content),
                lexeme,
                self.line,
                start_column,
            ),
            Some(mut i) => {
                i.string = content;
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
        let mut content = String::new();

        loop {
            let ch = self.peek();

            match ch {
                Some('\\') => {
                    self.advance(); // consume backslash
                    // Check if next char is a separator - if so, escape it
                    if let Some(next) = self.peek() {
                        if INLINE_STRING_SEPARATOR.contains(next) {
                            // Allow escaping separator characters in inline strings
                            content.push(next);
                            self.advance();
                        } else {
                            // Process standard escape sequence
                            let escaped = self.process_escape_sequence(start_column)?;
                            content.push(escaped);
                        }
                    } else {
                        return Err(ParseError::UnterminatedString {
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                Some(s) if INLINE_STRING_SEPARATOR.contains(s) => {
                    break;
                }
                Some(c) => {
                    content.push(c);
                    self.advance();
                }
                None => {
                    break; // inline string is always terminating
                }
            }
        }

        let lexeme: String = self.input[start..self.current].iter().collect();

        Ok(Token::new(
            TokenType::String(content),
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
            "let" => Token::new(TokenType::Let, lexeme, self.line, start_column),
            "match" => Token::new(TokenType::Match, lexeme, self.line, start_column),
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
