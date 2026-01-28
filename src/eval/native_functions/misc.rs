use crate::{
    count_args, define_help, define_native, eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, NativeContext, SpecialBoundClosure, SpecialFn, Value},
        EvalResult,
    }, parser::Expression, special_b_op
};
use std::rc::Rc;

// ============================================================================
// Misc helper functions
// ============================================================================

// If got null, convert to the value on the right side
// Usage: null $> or? 10
special_b_op!(IfNull, "or?", [on_null, val], ctx, {
    let val = ctx.inter.eval_expand(val, &ctx.env)?;

    Ok(match val.as_ref() {
        Value::Null => Rc::clone(&ctx.inter.eval_expand(on_null, ctx.env)?),
        _ => Rc::clone(&val),
    })
});

// ============================================================================
// Module Binding
// ============================================================================

pub fn bind_misc_module(env: &EnvRef, inter: Rc<Interpretator>) {
    env.define_help("misc.".to_string(), "miscelanios functions:

- or?: returns second value if first is null".to_string());
    define_native!(IfNull, env, inter);
    define_help!(IfNull, env, "[default:value value]: returns second value if first is null, otherwise returns first\n\nnull $> or? 10  ; 10\n5 $> or? 10  ; 5");
}
