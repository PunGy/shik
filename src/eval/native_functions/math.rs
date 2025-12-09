use crate::{
    count_args,
    eval::{
        error::RuntimeError,
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
use std::rc::Rc;

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
