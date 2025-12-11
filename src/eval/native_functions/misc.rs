use crate::{
    count_args, define_native,
    eval::{
        error::{RuntimeError, ShikError},
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeContext, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
use std::rc::Rc;

// ============================================================================
// Misc helper functions
// ============================================================================


// If got null, convert to the value on the right side
// Usage: null $> or? 10
native_op!(IfNull, "or?", [if_null, val], {
    Ok(match val.as_ref() {
        Value::Null => Rc::clone(if_null),
        _ => Rc::clone(val),
    })
});

// ============================================================================
// Module Binding
// ============================================================================

pub fn bind_misc_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Shell execution
    define_native!(IfNull, env, inter);
}
