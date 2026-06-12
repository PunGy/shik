use crate::eval::{error::RuntimeError, value::ValueRef};

pub mod error;
pub mod evaluator;
pub mod native_functions;
pub mod utils;
pub mod value;

pub type EvalResult = Result<ValueRef, RuntimeError>;
