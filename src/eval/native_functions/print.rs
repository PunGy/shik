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

native_op!(Print, "print", [arg], {
    println!("{}", arg);

    native_result(Value::Null)
});

pub fn bind_print_module(env: &EnvRef) {
    Print::define(env);
}
