/// Source location information for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn unknown() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 && self.column == 0 {
            write!(f, "unknown location")
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub expression: Expression,
    pub line: usize,
    pub column: usize,
}

/// Expression with source location information
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn with_span(kind: ExpressionKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            span: Span::new(line, column),
        }
    }

    // Convenience constructors that create expressions with unknown span
    // These are used in tests and places where span isn't critical
    pub fn number(value: f64) -> Self {
        Self::new(ExpressionKind::Number(value), Span::unknown())
    }

    pub fn string(value: String) -> Self {
        Self::new(ExpressionKind::String(value), Span::unknown())
    }

    pub fn identifier(name: String) -> Self {
        Self::new(ExpressionKind::Identifier(name), Span::unknown())
    }

    pub fn pipe(left: Expression, right: Expression) -> Self {
        Self::new(
            ExpressionKind::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::unknown(),
        )
    }

    pub fn chain(left: Expression, right: Expression) -> Self {
        Self::new(
            ExpressionKind::Chain {
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::unknown(),
        )
    }

    pub fn flow(left: Expression, right: Expression) -> Self {
        Self::new(
            ExpressionKind::Flow {
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::unknown(),
        )
    }

    pub fn application(function: Expression, argument: Expression) -> Self {
        Self::new(
            ExpressionKind::Application {
                function: Box::new(function),
                argument: Box::new(argument),
            },
            Span::unknown(),
        )
    }

    pub fn list(items: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::List(items), Span::unknown())
    }

    pub fn object(items: Vec<ObjectItem>) -> Self {
        Self::new(ExpressionKind::Object(items), Span::unknown())
    }

    pub fn parenthesized(expr: Expression) -> Self {
        Self::new(
            ExpressionKind::Parenthesized(Box::new(expr)),
            Span::unknown(),
        )
    }

    pub fn block(expressions: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Block(expressions), Span::unknown())
    }

    pub fn lazy(expressions: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Lazy(expressions), Span::unknown())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    // Literals
    Number(f64),
    String(String),
    StringInterpolation(StringInterpolationInfo),
    Identifier(String),

    // Binary operations
    Pipe {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Flow {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Chain {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Application {
        function: Box<Expression>,
        argument: Box<Expression>,
    },

    // Collections
    List(Vec<Expression>),
    Object(Vec<ObjectItem>),

    // Special forms
    Let {
        pattern: MatchPattern,
        value: Box<Expression>,
    },
    Lambda {
        parameters: Vec<MatchPattern>,
        rest: Option<String>,
        body: Box<Expression>,
    },
    Match {
        item: Box<Expression>,
        entries: Vec<MatchItem>,
    },

    // Grouping
    Parenthesized(Box<Expression>),
    Block(Vec<Expression>),
    Lazy(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringInterpolationInfo {
    // Interpolated string, with all interpolations replaced with placeholder "_"
    pub string: String,
    pub entries: Vec<Interpolation>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Interpolation {
    pub expression: Expression,
    pub start: usize,
    pub end: usize,

    // position of interpolation in the wrapped string
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectItem {
    pub key: Expression,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchItem {
    pub pattern: MatchPattern,
    pub resolve: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Identifier(String),
    Literal(LiteralPattern),
    List {
        patterns: Vec<MatchPattern>,
        rest: Option<String>,
    },
    NamedWildcard(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralPattern {
    Number(f64),
    String(String),
}
