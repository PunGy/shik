use crate::{
    count_args,
    eval::{
        error::{RuntimeError},
        evaluator::Interpretator,
        value::{EnvRef, NativeContext, SpecialClosure, NativeClosure, NativeFn, Value, ValueRef, SpecialFn},
        EvalResult,
    },
    parser::{Expression},
    native_op,
    define_native,
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
        _ => return Err(RuntimeError::InvalidApplication)
    }
});

native_op!(Set, "set", [name, val], ctx, {
    let name = name.expect_string()?;

    ctx.env.assign(name, Rc::clone(&val));
    Ok(Rc::clone(val))
});

pub fn bind_variable_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Var, env, inter);
    define_native!(Set, env, inter);
}
