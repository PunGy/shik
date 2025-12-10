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

native_op!(ListLen, "list.len", [lst], {
    let lst = lst.expect_list()?;

    native_result(Value::Number(lst.len() as f64))
});

pub fn bind_list_module(env: &EnvRef) {
    ListLen::define(&env);
}
