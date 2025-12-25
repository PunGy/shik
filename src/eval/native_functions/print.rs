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
};
use std::rc::Rc;

native_op!(Print, "print", [arg], {
    println!("{}", arg.to_string());

    native_result(Value::Null)
});

pub fn bind_print_module(env: &EnvRef, inter: Rc<Interpretator>) {
    Print::define(env, inter);
}
