use crate::{eval::value::ValueType, parser::Expression};
#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    NotYetImplemented(Expression),

    MissmatchedTypes { got: ValueType, expected: ValueType },
    InvalidApplication,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(var) => {
                write!(f, "Undefined variable '{}'", var,)
            }
            RuntimeError::MissmatchedTypes { got, expected } => {
                write!(
                    f,
                    "Missmatched types: expected {:?}, got {:?}",
                    expected, got
                )
            }
            RuntimeError::NotYetImplemented(expr) => {
                write!(f, "Feature {:?} still not implemeted", expr)
            }

            RuntimeError::InvalidApplication => {
                write!(
                    f,
                    "The application you are trying perform for some reason invalid..."
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
