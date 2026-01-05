use crate::{
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, NativeContext, SpecialClosure, SpecialFn, Value},
        EvalResult,
    },
    parser::Expression,
    special_op,
    define_native,
    define_help,
};
use std::rc::Rc;

// ============================================================================
// Misc helper functions
// ============================================================================

// If got null, convert to the value on the right side
// Usage: null $> or? 10
special_op!(IfNull, "or?", args, ctx, {
    let mut args_it = args.into_iter();

    let on_null = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
    let val = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
    let val = ctx.inter.eval_expr(val, &ctx.env)?;

    Ok(match val.as_ref() {
        Value::Null => Rc::clone(&ctx.inter.eval_expr(on_null, ctx.env)?),
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
