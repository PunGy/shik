use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::{native_result, number::Minus, polymorphic::PPlus},
        value::{
            EnvRef, NativeClosure, NativeContext, NativeFn, SpecialBoundClosure, SpecialFn, Value,
            ValueRef,
        },
        EvalResult,
    },
    native_op,
    parser::{Expression, ExpressionKind},
    special_b_op,
};
use std::rc::Rc;

special_b_op!(Let, "let", [name, val], ctx, {
    match &name.kind {
        ExpressionKind::Identifier(name) => {
            let val = ctx.inter.eval_expand(val, ctx.env)?;
            ctx.env.define(name.to_string(), Rc::clone(&val));
            Ok(val)
        }
        _ => {
            return Err(RuntimeError::invalid_application(
                "(let) variable name must be an identifier".to_string(),
            ))
        }
    }
});

native_op!(VarGet, "var.get", [name], ctx, {
    let name = name.expect_string()?;
    let var = ctx.env.lookup(name);

    match var {
        Some(var) => Ok(var),
        _ => native_result(Value::Null),
    }
});

special_b_op!(Set, "set", [name, val], ctx, {
    match &name.kind {
        ExpressionKind::Identifier(name) => {
            let val = ctx.inter.eval_expand(val, ctx.env)?;
            ctx.env.assign(name, Rc::clone(&val));
            Ok(val)
        }
        _ => {
            return Err(RuntimeError::invalid_application(
                "(set) variable name must be an identifier".to_string(),
            ))
        }
    }
});

special_b_op!(SetPlus, "set+", [name, val], ctx, {
    match &name.kind {
        ExpressionKind::Identifier(name) => {
            let current_val = ctx
                .env
                .lookup(name)
                .ok_or(RuntimeError::undefined_variable(name.clone()))?;
            let val = ctx.inter.eval_expand(val, ctx.env)?;

            let next_val = PPlus::run(&current_val, &val)?;
            ctx.env.assign(name, Rc::clone(&next_val));
            Ok(next_val)
        }
        _ => {
            return Err(RuntimeError::invalid_application(
                "(set+) variable name must be an identifier".to_string(),
            ))
        }
    }
});
special_b_op!(SetMinus, "set-", [name, val], ctx, {
    match &name.kind {
        ExpressionKind::Identifier(name) => {
            let current_val = ctx
                .env
                .lookup(name)
                .ok_or(RuntimeError::undefined_variable(name.clone()))?;
            let val = ctx.inter.eval_expand(val, ctx.env)?;

            let next_val = Minus::run(&val, &current_val)?;
            ctx.env.assign(name, Rc::clone(&next_val));
            Ok(next_val)
        }
        _ => {
            return Err(RuntimeError::invalid_application(
                "(set-) variable name must be an identifier".to_string(),
            ))
        }
    }
});

pub fn bind_variable_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "var.".to_string(),
        "variable module:

- let: define variable
- let$: define variable with pattern matching
- set: mutate value of the variable
- set+: mutate value of the variable by making concatenation
- set-: mutate value of the variable by making substraction
- var.get: gets variable value by name string"
            .to_string(),
    );

    define_native!(Let, env, inter);
    define_help!(Let, env, "[name:identifier value]: defines a new variable in current scope\n\nlet x 42\nlet name :Alice\nlet files $ file.list :./");

    define_native!(Set, env, inter);
    define_help!(
        Set,
        env,
        "[name:identifier value]: assigns new value to existing variable\n\nlet x 1\nset x 2"
    );

    define_native!(SetPlus, env, inter);
    define_help!(
        SetPlus,
        env,
        "[name:identifier value]: assigns new value with concatenation\n\nlet x 1\nset+ x 2\nprint x ;; 3"
    );

    define_native!(SetMinus, env, inter);
    define_help!(
        SetMinus,
        env,
        "[name:identifier value]: assigns new value with substraction\n\nlet x 3\nset- x 2\nprint x ;; 1"
    );

    define_native!(VarGet, env, inter);
    define_help!(VarGet, env, "[name:string]: gets variable value by name string, returns null if not found\n\nvar.get \"x\"");
}
