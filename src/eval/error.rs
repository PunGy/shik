use crate::{
    eval::value::ValueType,
    parser::{Expression, Span},
};

/// A runtime error with optional source location information
#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Option<Span>,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        Self { kind, span: None }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }

    // Convenience constructors
    pub fn undefined_variable(name: String) -> Self {
        Self::new(RuntimeErrorKind::UndefinedVariable(name))
    }

    pub fn mismatched_types(got: ValueType, expected: ValueType) -> Self {
        Self::new(RuntimeErrorKind::MissmatchedTypes { got, expected })
    }

    pub fn invalid_application(details: String) -> Self {
        Self::new(RuntimeErrorKind::InvalidApplication(details))
    }

    pub fn invalid_pattern_matching() -> Self {
        Self::new(RuntimeErrorKind::InvalidPatternMatching)
    }

    pub fn index_out_of_bounds(index: usize) -> Self {
        Self::new(RuntimeErrorKind::IndexOutOfBounds { index })
    }

    pub fn environment_dropped() -> Self {
        Self::new(RuntimeErrorKind::EnvironmentDropped)
    }

    pub fn not_yet_implemented(expr: Expression) -> Self {
        Self::new(RuntimeErrorKind::NotYetImplemented(expr))
    }

    pub fn custom(title: String, msg: String) -> Self {
        Self::new(RuntimeErrorKind::Custom(ShikError { title, msg }))
    }

    pub fn default_error(msg: String) -> Self {
        Self::custom("RuntimeError".to_string(), msg)
    }
}

#[derive(Debug)]
pub struct ShikError {
    pub title: String,
    pub msg: String,
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
    UndefinedVariable(String),
    NotYetImplemented(Expression),

    MissmatchedTypes {
        got: ValueType,
        expected: ValueType,
    },
    InvalidApplication(String),
    InvalidPatternMatching,
    IndexOutOfBounds {
        index: usize,
    },

    /// Environment was dropped - closure outlived its captured scope.
    /// This typically indicates a bug in the interpreter or an unusual
    /// pattern where a closure escapes its defining scope.
    EnvironmentDropped,

    Custom(ShikError),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // First write the error message
        match &self.kind {
            RuntimeErrorKind::UndefinedVariable(var) => {
                write!(f, "UndefinedVariable: Variable '{}' is not defined", var)?;
            }
            RuntimeErrorKind::MissmatchedTypes { got, expected } => {
                write!(f, "TypeError: Expected {:?}, but got {:?}", expected, got)?;
            }
            RuntimeErrorKind::NotYetImplemented(expr) => {
                write!(
                    f,
                    "NotImplemented: Feature {:?} is not yet implemented",
                    expr.kind
                )?;
            }
            RuntimeErrorKind::IndexOutOfBounds { index } => {
                write!(f, "IndexError: Index {} is out of bounds", index)?;
            }
            RuntimeErrorKind::InvalidApplication(details) => {
                write!(
                    f,
                    "ApplicationError: Cannot perform application - {details}"
                )?;
            }
            RuntimeErrorKind::InvalidPatternMatching => {
                write!(f, "PatternError: Unable to match the pattern")?;
            }
            RuntimeErrorKind::EnvironmentDropped => {
                write!(
                    f,
                    "InternalError: The closure's captured environment was garbage collected. \
                     This usually means a closure outlived its defining scope."
                )?;
            }
            RuntimeErrorKind::Custom(err) => {
                write!(f, "{}: {}", err.title, err.msg)?;
            }
        }

        // Then write the location if available
        if let Some(span) = &self.span {
            if span.line > 0 {
                write!(f, "\n  --> at {}", span)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

// Implement From for backward compatibility with code that creates errors directly
impl From<RuntimeErrorKind> for RuntimeError {
    fn from(kind: RuntimeErrorKind) -> Self {
        Self::new(kind)
    }
}

// Helper macro for creating errors with location
#[macro_export]
macro_rules! runtime_error {
    ($kind:expr, $span:expr) => {
        RuntimeError::new($kind).with_span($span)
    };
    ($kind:expr) => {
        RuntimeError::new($kind)
    };
}
