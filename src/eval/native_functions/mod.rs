pub mod bool;
pub mod math;
pub mod print;
pub mod keywords;
pub mod macros;

use crate::eval::{value::Value, EvalResult};
use std::rc::Rc;

pub fn native_result(val: Value) -> EvalResult {
    Ok(Rc::new(val))
}
