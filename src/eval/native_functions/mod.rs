pub mod bool;
pub mod math;
pub mod print;
pub mod keywords;
pub mod branching;
pub mod macros;
pub mod file;
pub mod string;
pub mod list;
pub mod variables;
pub mod shell;
pub mod misc;

use crate::eval::{value::Value, EvalResult};
use std::rc::Rc;

pub fn native_result(val: Value) -> EvalResult {
    Ok(Rc::new(val))
}
