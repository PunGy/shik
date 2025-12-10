use crate::{
    count_args,
    eval::{
        error::{RuntimeError},
        evaluator::Interpretator,
        value::{EnvRef, NativeContext, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
    define_native,
};
use std::rc::Rc;

native_op!(Var, "var", [name, val], ctx, {
    let name = name.expect_string()?;

    ctx.env.define(name.to_string(), Rc::clone(&val));
    Ok(Rc::clone(&val))
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
