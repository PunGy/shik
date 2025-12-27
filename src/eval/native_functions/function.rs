use crate::{
    count_args, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{
            EnvRef, ValueRef, NativeClosure, NativeContext, NativeFn, SpecialBoundClosure, SpecialFn, Value,
        },
        EvalResult,
    },
    native_op,
    parser::Expression,
    special_b_op,
};
use std::rc::Rc;

special_b_op!(Quote, ["'", "fn.quote"], [val], ctx, {
    ctx.inter.ctx.borrow_mut().quoted = true;
    let val = ctx.inter.eval_expr(val, ctx.env)?;
    ctx.inter.ctx.borrow_mut().quoted = false;

    Ok(Rc::clone(&val))
});

native_op!(Id, ["fn.id", "fn.invoke", "invoke"], [fun], ctx, {
    ctx.inter.ctx.borrow_mut().quoted = true;
    let res = ctx.apply(&fun, &Rc::new(Value::Null));
    ctx.inter.ctx.borrow_mut().quoted = false;

    res
});

// native_op!(Call, "call", [fun], ctx, {
//     match fname.as_ref() {
//         Value::NativeLambda(fun) => {
//             ctx.apply()
//         }
//     }
// });

pub fn bind_function_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Id, env, inter);
    define_native!(Quote, env, inter);
}
