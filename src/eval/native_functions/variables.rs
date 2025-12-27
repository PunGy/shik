use crate::{
    count_args, define_native, eval::{
        error::RuntimeError, evaluator::Interpretator, native_functions::native_result, value::{EnvRef, NativeClosure, NativeContext, NativeFn, SpecialClosure, SpecialFn, Value, ValueRef}, EvalResult
    }, native_op, parser::Expression, special_op
};
use std::rc::Rc;

special_op!(Let, "let", args, ctx, {
    let mut args_it = args.into_iter();
    let name = args_it.next().ok_or(RuntimeError::InvalidApplication)?;

    let val = args_it.next().ok_or(RuntimeError::InvalidApplication)?;

    match name {
        Expression::Identifier(name) => {
            let val = ctx.inter.eval_expr(val, &ctx.env)?;
            ctx.env.define(name.to_string(), Rc::clone(&val));
            Ok(val)
        }
        _ => return Err(RuntimeError::InvalidApplication),
    }
});

native_op!(VarGet, "var.get", [name], ctx, {
    let name = name.expect_string()?;
    let var = ctx.env.lookup(name);

    match var {
        Some(var) => {
            Ok(var)
        }
        _ => native_result(Value::Null),
    }
});

special_op!(Set, "set", args, ctx, {
    let mut args_it = args.into_iter();
    let name = args_it.next().ok_or(RuntimeError::InvalidApplication)?;

    let val = args_it.next().ok_or(RuntimeError::InvalidApplication)?;

    match name {
        Expression::Identifier(name) => {
            let val = ctx.inter.eval_expr(val, &ctx.env)?;
            ctx.env.assign(name, Rc::clone(&val));
            Ok(val)
        }
        _ => return Err(RuntimeError::InvalidApplication),
    }
});

pub fn bind_variable_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Let, env, inter);
    define_native!(Set, env, inter);
    define_native!(VarGet, env, inter);
}
