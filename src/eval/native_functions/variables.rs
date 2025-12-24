use crate::{
    define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, NativeContext, SpecialClosure, SpecialFn, Value},
        EvalResult,
    },
    parser::Expression,
    special_op,
};
use std::rc::Rc;

special_op!(Var, "var", args, ctx, {
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
    define_native!(Var, env, inter);
    define_native!(Set, env, inter);
}
