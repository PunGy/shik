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

native_op!(Or, "or", [x, y], {
    let x = x.expect_bool()?;
    let y = y.expect_bool()?;

    native_result(Value::Bool(x || y))
});

native_op!(And, "and", [x, y], {
    let x = x.expect_bool()?;
    let y = y.expect_bool()?;

    native_result(Value::Bool(x && y))
});

pub fn bind_bool_module(env: &EnvRef) {
    Or::define(&env);
    And::define(&env);
}
