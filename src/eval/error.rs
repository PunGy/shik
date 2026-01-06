use crate::{eval::value::ValueType, parser::Expression};

#[derive(Debug)]
pub struct ShikError {
    title: String,
    msg: String,
}

impl ShikError {
    pub fn default_error(msg: String) -> RuntimeError {
        RuntimeError::Custom(Self {
            title: "RuntimeError".to_string(),
            msg,
        })
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    NotYetImplemented(Expression),

    MissmatchedTypes { got: ValueType, expected: ValueType },
    InvalidApplication,
    InvalidPatternMatching,
    IndexOutOfBounds { index: usize },
    
    /// Environment was dropped - closure outlived its captured scope.
    /// This typically indicates a bug in the interpreter or an unusual
    /// pattern where a closure escapes its defining scope.
    EnvironmentDropped,

    Custom(ShikError),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(var) => {
                write!(f, "EvaluationError: Undefined variable '{}'", var,)
            }
            RuntimeError::MissmatchedTypes { got, expected } => {
                write!(
                    f,
                    "EvaluationError: Missmatched types: expected {:?}, got {:?}",
                    expected, got
                )
            }
            RuntimeError::NotYetImplemented(expr) => {
                write!(
                    f,
                    "EvaluationError: Feature {:?} still not implemeted",
                    expr
                )
            }

            RuntimeError::IndexOutOfBounds {index} => {
                write!(
                    f,
                    "IndexOutOfBounds: the index {} is out of bound",
                    index
                )
            }

            RuntimeError::InvalidApplication => {
                write!(
                    f,
                    "EvaluationError: The application you are trying perform for some reason invalid..."
                )
            }
            RuntimeError::InvalidPatternMatching => {
                write!(
                    f,
                    "PatternMatching: Unable to match the pattern"
                )
            }
            RuntimeError::EnvironmentDropped => {
                write!(
                    f,
                    "EnvironmentDropped: The closure's captured environment was garbage collected. \
                     This usually means a closure outlived its defining scope."
                )
            }
            RuntimeError::Custom(err) => {
                write!(f, "{}: {}", err.title, err.msg,)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
