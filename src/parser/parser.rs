use crate::parser::ast::*;
use crate::parser::error::ParseError;
use crate::parser::tokens::{Token, TokenType};
use std::collections::VecDeque;

pub type ParseResult<T> = Result<T, ParseError>;

/// Precedence levels for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 0,
    Pipe = 1,  // $> - pipe/apply value to function (lowest precedence of operators)
    Chain = 2, // $ - chain application - acts like apply, but with lower precedence
    Flow = 3,  // #> - function composition (medium precedence)
    Apply = 4, // function application (highest precedence)
}

pub struct Parser {
    tokens: VecDeque<Token>,
    current: Option<Token>,
    peek: Option<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = VecDeque::from(tokens);
        tokens.retain(|t| {
            !matches!(
                t.token_type,
                TokenType::BlockComment | TokenType::SingleLineComment
            )
        });

        let mut parser = Parser {
            tokens,
            current: None,
            peek: None,
        };

        parser.init();

        // Skip initial newlines
        while parser.is_newline() {
            parser.advance();
        }

        parser
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines between statements
            while self.is_newline() {
                self.advance();
            }

            if self.is_at_end() {
                break;
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Consume optional newline after statement
            if self.is_newline() {
                self.advance();
            }
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        let line = self.current_line();
        let column = self.current_column();

        let expression = self.parse_expression(Precedence::Lowest)?;

        Ok(Statement {
            expression,
            line,
            column,
        })
    }

    fn current_span(&self) -> Span {
        Span::new(self.current_line(), self.current_column())
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        // Parse prefix/primary expression
        let mut left = self.parse_primary()?;

        // Parse infix expressions
        loop {
            if self.is_at_end() {
                break;
            }

            let should_continue = match self.current_token_type() {
                Ok(TokenType::Pipe) if precedence < Precedence::Pipe => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after pipe operator
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Pipe)?;
                    left = Expression::new(
                        ExpressionKind::Pipe {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Chain) if precedence < Precedence::Chain => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after chain operator
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Chain)?;
                    left = Expression::new(
                        ExpressionKind::Chain {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Flow) if precedence < Precedence::Flow => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after flow operator
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Flow)?;
                    left = Expression::new(
                        ExpressionKind::Flow {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Newline) => {
                    // Newline breaks the expression
                    false
                }
                _ => {
                    if self.can_start_primary() && precedence < Precedence::Apply {
                        let span = left.span;
                        let arg = self.parse_expression(Precedence::Apply)?;
                        left = Expression::new(
                            ExpressionKind::Application {
                                function: Box::new(left),
                                argument: Box::new(arg),
                            },
                            span,
                        );
                        true
                    } else {
                        false
                    }
                }
            };

            if !should_continue {
                break;
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        if self.is_at_end() {
            return Err(ParseError::UnexpectedEndOfInput {
                expected: "primary expression".to_string(),
            });
        }

        let span = self.current_span();

        // Use reference to avoid cloning the token
        let token_type = self.current_token_type_ref();

        match token_type {
            Some(TokenType::Number(n)) => {
                let value = *n;
                self.advance();
                Ok(Expression::new(ExpressionKind::Number(value), span))
            }
            Some(TokenType::String(s)) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::new(ExpressionKind::String(value), span))
            }
            Some(TokenType::StringInterpolation(info)) => {
                let value = info.clone();
                self.advance();
                let interpolatons = value
                    .entries
                    .into_iter()
                    .map(|i| {
                        let mut subparser = Parser::new(i.tokens);
                        subparser.parse().map(|mut prg| Interpolation {
                            expression: prg.statements.swap_remove(0).expression,
                            start: i.start,
                            end: i.end,
                            position: i.position,
                        })
                    })
                    .collect::<ParseResult<Vec<Interpolation>>>()?;
                let inter_info = StringInterpolationInfo {
                    string: value.string,
                    entries: interpolatons,
                };
                Ok(Expression::new(
                    ExpressionKind::StringInterpolation(inter_info),
                    span,
                ))
            }
            Some(TokenType::Ident) => {
                let name = self.current_lexeme().to_string();
                self.advance();
                Ok(Expression::new(ExpressionKind::Identifier(name), span))
            }
            Some(TokenType::Let) => {
                self.advance();
                self.parse_let_expression(span)
            }
            Some(TokenType::Fn) => {
                self.advance();
                self.parse_lambda(span)
            }
            Some(TokenType::Match) => {
                self.advance();
                self.parse_match_expression(span)
            }
            Some(TokenType::LeftParen) => {
                self.advance();
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.expect_token(TokenType::RightParen)?;
                Ok(Expression::new(
                    ExpressionKind::Parenthesized(Box::new(expr)),
                    span,
                ))
            }
            Some(TokenType::OpenBlock) => {
                self.advance();
                self.parse_block(span)
            }
            Some(TokenType::OpenLazy) => {
                self.advance();
                self.parse_lazy(span)
            }
            Some(TokenType::LeftBracket) => {
                self.advance();
                self.parse_list(span)
            }
            Some(TokenType::LeftCurlyBracket) => {
                self.advance();
                self.parse_object(span)
            }
            Some(TokenType::Newline) => {
                self.advance();
                self.parse_primary()
            }
            _ => {
                let token = self.current_token_cloned()?;
                Err(ParseError::unexpected_token(
                    token,
                    "expression".to_string(),
                ))
            }
        }
    }

    fn parse_let_expression(&mut self, span: Span) -> ParseResult<Expression> {
        let pattern = self.parse_match_pattern()?;
        let value = Box::new(self.parse_expression(Precedence::Lowest)?);

        Ok(Expression::new(
            ExpressionKind::Let { pattern, value },
            span,
        ))
    }

    fn parse_match_expression(&mut self, span: Span) -> ParseResult<Expression> {
        let item = Box::new(self.parse_primary()?);

        let mut entries: Vec<MatchItem> = Vec::new();

        self.expect_token(TokenType::LeftCurlyBracket)?;

        while !self.check_token(&TokenType::RightCurlyBracket) && !self.is_at_end() {
            if self.is_newline() {
                self.advance();
                continue;
            }

            let pattern = self.parse_match_pattern()?;
            let resolve = self.parse_primary()?;

            entries.push(MatchItem { pattern, resolve });
        }

        self.expect_token(TokenType::RightCurlyBracket)?;

        Ok(Expression::new(
            ExpressionKind::Match { item, entries },
            span,
        ))
    }

    fn parse_lambda(&mut self, span: Span) -> ParseResult<Expression> {
        self.expect_token(TokenType::LeftBracket)?;

        let mut parameters = Vec::new();
        let mut rest = None;

        while !self.check_token(&TokenType::RightBracket) {
            if self.check_token(&TokenType::Hash) {
                self.advance();
                rest = Some(self.expect_identifier()?);
                break;
            }
            parameters.push(self.parse_match_pattern()?);
        }

        self.expect_token(TokenType::RightBracket)?;

        // Parse the body - this should parse the entire remaining expression
        let body = Box::new(self.parse_expression(Precedence::Lowest)?);

        Ok(Expression::new(
            ExpressionKind::Lambda {
                parameters,
                rest,
                body,
            },
            span,
        ))
    }

    fn parse_match_pattern(&mut self) -> ParseResult<MatchPattern> {
        match self.current_token_type_ref() {
            Some(TokenType::Ident) => {
                let name = self.current_lexeme().to_string();
                self.advance();

                if name == "_" {
                    Ok(MatchPattern::Wildcard)
                } else {
                    Ok(MatchPattern::Identifier(name))
                }
            }
            Some(TokenType::Number(n)) => {
                let value = *n;
                self.advance();
                Ok(MatchPattern::Literal(LiteralPattern::Number(value)))
            }
            Some(TokenType::String(s)) => {
                let value = s.clone();
                self.advance();
                Ok(MatchPattern::Literal(LiteralPattern::String(value)))
            }
            Some(TokenType::LeftBracket) => {
                self.advance();
                let mut patterns = Vec::new();
                let mut rest = None;

                while !self.check_token(&TokenType::RightBracket) {
                    if self.check_token(&TokenType::Hash) {
                        self.advance();
                        rest = Some(self.expect_identifier()?);
                        break;
                    }
                    patterns.push(self.parse_match_pattern()?);
                }

                self.expect_token(TokenType::RightBracket)?;
                Ok(MatchPattern::List { patterns, rest })
            }
            Some(TokenType::Hash) => {
                self.advance();
                let token = self.expect_token(TokenType::Ident)?;
                let name = token.lexeme.to_string();

                Ok(MatchPattern::NamedWildcard(name))
            }
            _ => {
                let token = self.current_token_cloned()?;
                Err(ParseError::unexpected_token(
                    token,
                    "match pattern".to_string(),
                ))
            }
        }
    }

    fn parse_block(&mut self, span: Span) -> ParseResult<Expression> {
        let expressions = self.parse_block_contents()?;
        self.expect_token(TokenType::RightParen)?;
        Ok(Expression::new(ExpressionKind::Block(expressions), span))
    }

    fn parse_lazy(&mut self, span: Span) -> ParseResult<Expression> {
        let expressions = self.parse_block_contents()?;
        self.expect_token(TokenType::RightParen)?;
        Ok(Expression::new(ExpressionKind::Lazy(expressions), span))
    }

    /// Shared parsing logic for block and lazy expressions.
    /// Handles newline-separated statements where each line can contain
    /// function applications and operators.
    fn parse_block_contents(&mut self) -> ParseResult<Vec<Expression>> {
        let mut expressions = Vec::new();
        let mut current_line_expr: Option<Expression> = None;
        let mut has_newlines = false;

        while !self.check_token(&TokenType::RightParen) && !self.is_at_end() {
            // Check for newlines
            if self.is_newline() {
                has_newlines = true;
                // If we have an expression on the current line, finalize it
                if let Some(expr) = current_line_expr.take() {
                    expressions.push(expr);
                }
                self.advance();
                continue;
            }

            // Parse a primary expression
            let primary = self.parse_primary()?;

            // Build up the current line expression
            current_line_expr = Some(match current_line_expr.take() {
                None => primary,
                Some(left) => {
                    let span = left.span;
                    Expression::new(
                        ExpressionKind::Application {
                            function: Box::new(left),
                            argument: Box::new(primary),
                        },
                        span,
                    )
                }
            });

            // Check if there's an operator that continues the expression
            // The operator should bind to the entire expression built so far
            if matches!(
                self.current_token_type(),
                Ok(TokenType::Pipe) | Ok(TokenType::Flow) | Ok(TokenType::Chain)
            ) {
                // Continue parsing with the accumulated expression as the left side
                let left = current_line_expr.take().unwrap();
                let full_expr = self.continue_expression(left, Precedence::Lowest)?;

                if has_newlines {
                    current_line_expr = Some(full_expr);
                } else {
                    // No newlines in block yet, each operator-terminated expression is separate
                    // But we should continue building if more primaries follow
                    current_line_expr = Some(full_expr);
                }
            }
        }

        // Handle any remaining expression on the last line
        if let Some(expr) = current_line_expr {
            expressions.push(expr);
        }

        Ok(expressions)
    }

    fn continue_expression(
        &mut self,
        mut left: Expression,
        precedence: Precedence,
    ) -> ParseResult<Expression> {
        // Continue parsing an expression with operators
        loop {
            if self.is_at_end() {
                break;
            }

            let should_continue = match self.current_token_type() {
                Ok(TokenType::Pipe) if precedence < Precedence::Pipe => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after pipe operator (continuation)
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Pipe)?;
                    left = Expression::new(
                        ExpressionKind::Pipe {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Chain) if precedence < Precedence::Chain => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after chain operator (continuation)
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Chain)?;
                    left = Expression::new(
                        ExpressionKind::Chain {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Flow) if precedence < Precedence::Flow => {
                    let span = left.span;
                    self.advance();
                    // Allow newlines after flow operator (continuation)
                    while self.is_newline() {
                        self.advance();
                    }
                    let right = self.parse_expression(Precedence::Flow)?;
                    left = Expression::new(
                        ExpressionKind::Flow {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                        span,
                    );
                    true
                }
                Ok(TokenType::Newline) => false,
                _ => false,
            };

            if !should_continue {
                break;
            }
        }

        Ok(left)
    }

    fn parse_list(&mut self, span: Span) -> ParseResult<Expression> {
        let mut items = Vec::new();

        while !self.check_token(&TokenType::RightBracket) && !self.is_at_end() {
            if self.is_newline() {
                self.advance();
                continue;
            }
            items.push(self.parse_primary()?);
        }

        self.expect_token(TokenType::RightBracket)?;
        Ok(Expression::new(ExpressionKind::List(items), span))
    }

    fn parse_object(&mut self, span: Span) -> ParseResult<Expression> {
        let mut items = Vec::new();

        while !self.check_token(&TokenType::RightCurlyBracket) && !self.is_at_end() {
            if self.is_newline() {
                self.advance();
                continue;
            }
            let key = self.parse_primary()?;
            let value = self.parse_primary()?;
            items.push(ObjectItem { key, value });
        }

        self.expect_token(TokenType::RightCurlyBracket)?;
        Ok(Expression::new(ExpressionKind::Object(items), span))
    }

    // Helper methods

    fn init(&mut self) {
        self.current = self.tokens.pop_front();
        self.peek = self.tokens.pop_front();
    }

    fn advance(&mut self) {
        self.current = self.peek.take();
        self.peek = self.tokens.pop_front();
    }

    /// Returns a reference to the current token type without cloning.
    fn current_token_type_ref(&self) -> Option<&TokenType> {
        self.current.as_ref().map(|t| &t.token_type)
    }

    /// Returns the current token's lexeme as a string slice.
    fn current_lexeme(&self) -> &str {
        self.current
            .as_ref()
            .map(|t| t.lexeme.as_str())
            .unwrap_or("")
    }

    /// Clones the current token. Use sparingly - prefer reference-based methods.
    fn current_token_cloned(&self) -> ParseResult<Token> {
        self.current
            .clone()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput {
                expected: "token".to_string(),
            })
    }

    fn current_token_type(&self) -> ParseResult<TokenType> {
        self.current_token_type_ref()
            .cloned()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput {
                expected: "token".to_string(),
            })
    }

    fn check_token(&self, token_type: &TokenType) -> bool {
        self.current
            .as_ref()
            .map(|t| std::mem::discriminant(&t.token_type) == std::mem::discriminant(token_type))
            .unwrap_or(false)
    }

    fn expect_token(&mut self, token_type: TokenType) -> ParseResult<Token> {
        if let Some(current_type) = self.current_token_type_ref() {
            if std::mem::discriminant(current_type) == std::mem::discriminant(&token_type) {
                let token = self.current_token_cloned()?;
                self.advance();
                return Ok(token);
            }
        }
        let token = self.current_token_cloned()?;
        Err(ParseError::unexpected_token(
            token,
            format!("{:?}", token_type),
        ))
    }

    fn expect_identifier(&mut self) -> ParseResult<String> {
        if let Some(TokenType::Ident) = self.current_token_type_ref() {
            let name = self.current_lexeme().to_string();
            self.advance();
            Ok(name)
        } else {
            let token = self.current_token_cloned()?;
            Err(ParseError::unexpected_token(
                token,
                "identifier".to_string(),
            ))
        }
    }

    fn can_start_primary(&self) -> bool {
        matches!(
            self.current.as_ref().map(|t| &t.token_type),
            Some(TokenType::Number(_))
                | Some(TokenType::String(_))
                | Some(TokenType::StringInterpolation(_))
                | Some(TokenType::Ident)
                | Some(TokenType::Let)
                | Some(TokenType::Fn)
                | Some(TokenType::LeftParen)
                | Some(TokenType::OpenBlock)
                | Some(TokenType::OpenLazy)
                | Some(TokenType::LeftBracket)
                | Some(TokenType::LeftCurlyBracket)
        )
    }

    fn is_at_end(&self) -> bool {
        self.current.is_none()
            || matches!(
                self.current.as_ref().map(|t| &t.token_type),
                Some(TokenType::Eof)
            )
    }

    fn is_newline(&self) -> bool {
        matches!(
            self.current.as_ref().map(|t| &t.token_type),
            Some(TokenType::Newline)
        )
    }

    fn current_line(&self) -> usize {
        self.current.as_ref().map(|t| t.line).unwrap_or(0)
    }

    fn current_column(&self) -> usize {
        self.current.as_ref().map(|t| t.column).unwrap_or(0)
    }
}
