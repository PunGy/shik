use crate::{
    count_args, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::{list::{ListAt, ListIterate}, native_result, number::Plus, string::{StringCharAt, StringConcat, StringIterate}},
        value::{EnvRef, NativeClosure, NativeContext, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
use std::rc::Rc;

native_op!(PPlus, "+", [x, y], {
    match (x.as_ref(), y.as_ref()) {
        (Value::String(_), Value::String(_)) => StringConcat::run(x, y),
        (Value::Number(_), Value::Number(_)) => Plus::run(x, y),

        (Value::String(_), other) => {
            StringConcat::run(x, &other.into_string())
        }
        (other, Value::String(_)) => {
            StringConcat::run(&other.into_string(), y)
        }

        (_, _) => return Err(RuntimeError::InvalidApplication),
    }
});

native_op!(At, "at", [inx, s], {
    match s.as_ref() {
        Value::String(_) => StringCharAt::run(inx, s),
        Value::List(_) => ListAt::run(inx, s),
        _ => Err(RuntimeError::InvalidApplication),
    }
});

native_op!(Iterate, "iterate", [func, s], ctx, {
    match s.as_ref() {
        Value::String(_) => StringIterate::run(func, s, ctx),
        Value::List(_) => ListIterate::run(func, s, ctx),
        _ => Err(RuntimeError::InvalidApplication),
    }
});

pub fn bind_poly_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(PPlus, env, inter);
    define_native!(At, env, inter);
    define_native!(Iterate, env, inter);
}
