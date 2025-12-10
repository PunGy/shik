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

native_op!(MakeString, "string", [x], {
    native_result(match x.as_ref() {
        Value::Number(n) => Value::String(n.to_string()),
        Value::String(s) => Value::String(s.clone()),
        Value::Object(_) => Value::String("[object]".to_string()),
        Value::List(_) => Value::String("[list]".to_string()),
        Value::Null => Value::String("[null]".to_string()),
        Value::Lambda(_) | Value::SpecialForm(_) | Value::NativeLambda(_) => {
            Value::String("[lambda]".to_string())
        }
        _ => Value::String("".to_string()),
    })
});

native_op!(StringSplit, "string.split", [with, str], {
    let str = str.expect_string()?;
    let with = with.expect_string()?;

    let res = str
        .split(with)
        .map(|s| Rc::new(Value::String(s.to_string())))
        .collect::<Vec<ValueRef>>();

    native_result(Value::List(res))
});

pub fn bind_string_module(env: &EnvRef) {
    MakeString::define(&env);
    StringSplit::define(&env);
}
