pub mod bool;
pub mod number;
pub mod polymorphic;
pub mod keywords;
pub mod branching;
pub mod macros;
pub mod file;
pub mod string;
pub mod list;
pub mod variables;
pub mod shell;
pub mod misc;
pub mod function;
pub mod help;

use crate::eval::{
    value::{Value, null_value, bool_value, number_value},
    EvalResult,
};
use std::rc::Rc;

/// Create a result from a Value, using cached values when possible
pub fn native_result(val: Value) -> EvalResult {
    match val {
        Value::Null => Ok(null_value()),
        Value::Bool(b) => Ok(bool_value(b)),
        Value::Number(n) => Ok(number_value(n)),
        _ => Ok(Rc::new(val)),
    }
}

/// Return cached null value
#[inline]
pub fn native_null() -> EvalResult {
    Ok(null_value())
}

/// Return cached boolean value
#[inline]
pub fn native_bool(b: bool) -> EvalResult {
    Ok(bool_value(b))
}

/// Return cached or new number value
#[inline]
pub fn native_number(n: f64) -> EvalResult {
    Ok(number_value(n))
}
