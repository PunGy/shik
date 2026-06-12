use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{
            EnvRef, NativeClosure, NativeContext, NativeFn, SpecialBoundClosure, SpecialFn, Value,
            ValueRef,
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
    let res = ctx.apply(fun, &Rc::new(Value::Null), ctx.env);
    ctx.inter.ctx.borrow_mut().quoted = false;

    res
});

pub fn bind_function_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "fn.".to_string(),
        "fn module:

- fn.id, fn.invoke, invoke: invokes lambda with null argument
- ', fn.quote: quotes the lambda (prevents evaluation)"
            .to_string(),
    );

    define_native!(Id, env, inter);
    define_help!(Id, env, "[fn:lambda]: invokes a lambda with null argument. Useful for callbacks\n\n[ (fn [] print :hello) (fn [] print :world) ] $> list.iterate fn.invoke");

    define_native!(Quote, env, inter);
    define_help!(Quote, env, "[fn:lambda]: quotes a lambda, preventing evaluation of identifiers\n\nlet say-hi fn [] print :hi\n\nlet hi say-hi ; hi = null, \"hi\" was printed\nlet hi (' say-hi) ; hi = lambda, nothing printed");
}
