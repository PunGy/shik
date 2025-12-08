use std::rc::Rc;

use crate::eval::{
    error::RuntimeError,
    value::{EnvRef, NativeClosure, NativeFn, Value, ValueRef, ValueType},
    EvalResult,
};

#[derive(Debug)]
pub struct Plus;
impl NativeFn for Plus {
    fn exec(&self, args: &Vec<ValueRef>) -> EvalResult {
        match args.as_slice() {
            [x, y] => {
                let x = x.as_number().ok_or(RuntimeError::MissmatchedTypes {
                    got: x.get_type(),
                    expected: ValueType::Number,
                })?;
                let y = y.as_number().ok_or(RuntimeError::MissmatchedTypes {
                    got: y.get_type(),
                    expected: ValueType::Number,
                })?;

                Ok(Rc::new(Value::Number(x + y)))
            }
            _ => Err(RuntimeError::InvalidApplication),
        }
    }
}


#[derive(Debug)]
pub struct Minus;
impl NativeFn for Minus {
    fn exec(&self, args: &Vec<ValueRef>) -> EvalResult {
        match args.as_slice() {
            [x, y] => {
                let x = x.as_number().ok_or(RuntimeError::MissmatchedTypes {
                    got: x.get_type(),
                    expected: ValueType::Number,
                })?;
                let y = y.as_number().ok_or(RuntimeError::MissmatchedTypes {
                    got: y.get_type(),
                    expected: ValueType::Number,
                })?;

                Ok(Rc::new(Value::Number(x - y)))
            }
            _ => Err(RuntimeError::InvalidApplication),
        }
    }
}

pub fn bind_math_module(env: &EnvRef) {
    env.define("+".to_string(), Rc::new(Value::NativeLambda(Rc::new(NativeClosure::new(2, Rc::new(Plus))))));
    env.define("-".to_string(), Rc::new(Value::NativeLambda(Rc::new(NativeClosure::new(2, Rc::new(Minus))))));
}

