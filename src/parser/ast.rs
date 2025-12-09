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

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
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
        pattern: LetPattern,
        value: Box<Expression>,
    },
    Lambda {
        parameters: Vec<MatchPattern>,
        rest: Option<String>,
        body: Box<Expression>,
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
pub enum LetPattern {
    Identifier(String),
    List {
        patterns: Vec<LetPattern>,
        rest: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Identifier(String),
    Literal(LiteralPattern),
    List {
        patterns: Vec<MatchPattern>,
        rest: Option<String>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralPattern {
    Number(f64),
    String(String),
}

impl Expression {
    pub fn number(value: f64) -> Self {
        Expression::Number(value)
    }

    pub fn string(value: String) -> Self {
        Expression::String(value)
    }

    pub fn identifier(name: String) -> Self {
        Expression::Identifier(name)
    }

    pub fn pipe(left: Expression, right: Expression) -> Self {
        Expression::Pipe {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn chain(left: Expression, right: Expression) -> Self {
        Expression::Chain {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn flow(left: Expression, right: Expression) -> Self {
        Expression::Flow {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn application(function: Expression, argument: Expression) -> Self {
        Expression::Application {
            function: Box::new(function),
            argument: Box::new(argument),
        }
    }

    pub fn list(items: Vec<Expression>) -> Self {
        Expression::List(items)
    }

    pub fn object(items: Vec<ObjectItem>) -> Self {
        Expression::Object(items)
    }

    pub fn parenthesized(expr: Expression) -> Self {
        Expression::Parenthesized(Box::new(expr))
    }

    pub fn block(expressions: Vec<Expression>) -> Self {
        Expression::Block(expressions)
    }

    pub fn lazy(expressions: Vec<Expression>) -> Self {
        Expression::Lazy(expressions)
    }
}
