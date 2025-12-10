use crate::{
    count_args,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeContext, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
    define_native,
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

native_op!(Eq, "=", [x, y], {
    native_result(Value::Bool(match (x.as_ref(), y.as_ref()) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }))
});
native_op!(Not, "not", [x], {
    let x = x.expect_bool()?;
    native_result(Value::Bool(!x))
});

native_op!(Gt, ">", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x > y))
});
native_op!(Lt, "<", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x < y))
});

pub fn bind_bool_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Eq, env, inter);
    define_native!(Gt, env, inter);
    define_native!(Lt, env, inter);
    define_native!(Not, env, inter);
    define_native!(Or, env, inter);
    define_native!(And, env, inter);
}
