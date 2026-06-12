pub mod bool;
pub mod branching;
pub mod file;
pub mod function;
pub mod help;
pub mod keywords;
pub mod list;
pub mod macros;
pub mod misc;
pub mod number;
pub mod object;
pub mod polymorphic;
pub mod shell;
pub mod string;
pub mod variables;

use crate::eval::{
    value::{bool_value, null_value, number_value, Value},
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
