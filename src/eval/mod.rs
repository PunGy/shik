use crate::eval::{error::RuntimeError, value::ValueRef};

pub mod error;
pub mod evaluator;
pub mod value;
pub mod native_functions;

pub type EvalResult = Result<ValueRef, RuntimeError>;
