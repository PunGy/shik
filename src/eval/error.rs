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

            RuntimeError::InvalidApplication => {
                write!(
                    f,
                    "EvaluationError: The application you are trying perform for some reason invalid..."
                )
            }
            RuntimeError::Custom(err) => {
                write!(f, "{}: {}", err.title, err.msg,)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
