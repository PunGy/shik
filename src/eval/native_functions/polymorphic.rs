use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        native_functions::{
            list::{ListAt, ListIterate, ListIterateBackward},
            number::Plus,
            string::{StringCharAt, StringConcat, StringIterate, StringIterateBackward},
        },
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

        (Value::String(_), other) => StringConcat::run(x, &other.into_string()),
        (other, Value::String(_)) => StringConcat::run(&other.into_string(), y),

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

native_op!(
    IterateBackward,
    ["<iterate", "iterate-backward"],
    [func, s],
    ctx,
    {
        match s.as_ref() {
            Value::String(_) => StringIterateBackward::run(func, s, ctx),
            Value::List(_) => ListIterateBackward::run(func, s, ctx),
            _ => Err(RuntimeError::InvalidApplication),
        }
    }
);

native_op!(Print, "print", [arg], {
    println!("{}", arg.to_string());

    native_result(Value::Null)
});

pub fn bind_poly_module(env: &EnvRef, inter: Rc<Interpretator>) {
    env.define_help(
        "general.".to_string(),
        "general module:
Contains polymorphic functions for values with similar behaviour.

- +: addition (number, string, string + any)
- iterate: forward iteration (string, list)
- <iterate, iterate-backward: backward iteration (string, list)
- at: gets element at index (string, list)"
            .to_string(),
    );

    define_native!(PPlus, env, inter);
    define_help!(PPlus, env, "[value value]: polymorphic addition - adds numbers or concatenates strings\n\n+ 2 3  ; 5\n+ \"hello\" \" world\"  ; \"hello world\"");

    define_native!(At, env, inter);
    define_help!(At, env, "[number value]: gets element at index from string or list\n\nat 0 \"hello\"  ; \"h\"\nat 1 [1 2 3]  ; 2");

    define_native!(Iterate, env, inter);
    define_help!(Iterate, env, "[lambda value]: iterates over string (chars) or list (elements)\n\niterate print \"abc\"  ; prints a, b, c");

    define_native!(IterateBackward, env, inter);
    define_help!(IterateBackward, env, "[lambda value]: iterates in reverse over string or list\n\n<iterate print [1 2 3]  ; prints 3, 2, 1");

    define_native!(Print, env, inter);
    define_help!(Print, env, "[value]: prints value to stdout with newline\n\nprint \"hello world\"");
}
