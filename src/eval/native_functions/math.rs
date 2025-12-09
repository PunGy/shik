use std::rc::Rc;

use crate::eval::{
    error::RuntimeError,
    value::{EnvRef, NativeClosure, NativeFn, Value, ValueRef, ValueType},
    EvalResult,
};

macro_rules! native_op {
    // Pattern: name, [arg1, arg2...], body
    ($name:ident, $fn_title:expr, [$($arg:ident),*], $body:expr) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            fn exec(&self, args: &Vec<ValueRef>) -> EvalResult {
                if args.len() != count_args!($($arg),*) {
                    return Err(RuntimeError::InvalidApplication)
                }
                let mut iter = args.iter();
                $(let $arg = iter.next().unwrap();)*

                $body
            }

        }

        impl $name {
            pub fn define(env: &EnvRef) {
                env.define(
                    ($fn_title).to_string(),
                    Rc::new(Value::NativeLambda(Rc::new(NativeClosure::new(
                        count_args!($($arg),*),
                        Rc::new($name),
                    )))),
                );
            }
        }
    };
}
macro_rules! count_args {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_args!($($tail),*) };
}

fn native_result(val: Value) -> EvalResult {
    Ok(Rc::new(val))
}

native_op!(Plus, "+", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x + y))
});

native_op!(Minus, "-", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x - y))
});

native_op!(Multiply, "*", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x * y))
});

native_op!(Divide, "/", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x / y))
});

pub fn bind_math_module(env: &EnvRef) {
    Plus::define(&env);
    Minus::define(&env);
    Divide::define(&env);
    Multiply::define(&env);
}
