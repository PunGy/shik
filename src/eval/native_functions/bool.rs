use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::{native_result, string::StringEq},
        value::{
            bool_value, EnvRef, NativeClosure, NativeContext, NativeFn, SpecialBoundClosure, SpecialFn, Value, ValueRef
        },
        EvalResult,
    },
    native_op,
    parser::Expression,
    special_b_op,
};
use std::rc::Rc;

native_op!(Bool, "bool", [val], {
    native_result(match val.as_ref() {
        Value::Number(val) => {
            if *val == 0.0 {
                Value::Bool(false)
            } else {
                Value::Bool(true)
            }
        }
        Value::Null => Value::Bool(false),
        Value::String(val) => {
            if val.is_empty() {
                Value::Bool(false)
            } else {
                Value::Bool(true)
            }
        }
        Value::List(val) => {
            if val.is_empty() {
                Value::Bool(false)
            } else {
                Value::Bool(true)
            }
        }
        Value::Object(val) => {
            if val.is_empty() {
                Value::Bool(false)
            } else {
                Value::Bool(true)
            }
        }
        _ => Value::Bool(true),
    })
});

special_b_op!(Or, "or", [x, y], ctx, {
    let x = ctx.inter.eval_expr(x, &ctx.env)?;
    let x = x.expect_bool()?;

    if x {
        return Ok(bool_value(true));
    }

    let y = ctx.inter.eval_expr(y, &ctx.env)?;
    let y = y.expect_bool()?;

    Ok(bool_value(y))
});

special_b_op!(And, "and", [x, y], ctx, {
    let x = ctx.inter.eval_expr(x, &ctx.env)?;
    let x = x.expect_bool()?;

    if !x {
        return Ok(bool_value(false));
    }

    let y = ctx.inter.eval_expr(y, &ctx.env)?;
    let y = y.expect_bool()?;

    return Ok(bool_value(y))
});

native_op!(Eq, "=", [x, y], {
    native_result(Value::Bool(match (x.as_ref(), y.as_ref()) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(_), Value::String(_)) => return StringEq::run(x, y),
        (Value::Null, Value::Null) => true,
        _ => false,
    }))
});
native_op!(NotEq, "!=", [x, y], {
    let res = Eq::run(x, y)?.expect_bool()?;
    native_result(Value::Bool(!res))
});
native_op!(Not, "not", [x], {
    let x = x.expect_bool()?;
    native_result(Value::Bool(!x))
});

native_op!(Gt, ">", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x > y))
});
native_op!(GtE, ">=", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x >= y))
});
native_op!(Lt, "<", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x < y))
});
native_op!(LtE, "<=", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Bool(x <= y))
});

pub fn bind_bool_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "bool.".to_string(),
        "bool module:

Comparison:
- =: equality check
- !=: inequality check
- >: greater than
- >=: greater than or equal
- <: less than
- <=: less than or equal

Logic:
- not: logical negation
- or: logical OR
- and: logical AND

Conversion:
- bool: convert value to boolean"
            .to_string(),
    );

    define_native!(Eq, env, inter);
    define_help!(Eq, env, "[value value]: checks equality of two values (numbers, bools, strings, or null)\n\n= 5 5  ; true");

    define_native!(NotEq, env, inter);
    define_help!(
        NotEq,
        env,
        "[value value]: checks inequality of two values\n\n!= 5 3  ; true"
    );

    define_native!(Gt, env, inter);
    define_help!(
        Gt,
        env,
        "[number number]: returns true if first number is greater than second\n\n> 5 3  ; true"
    );
    define_native!(GtE, env, inter);
    define_help!(
        GtE,
        env,
        "[number number]: returns true if first number is greater than second or equal\n\n>= 3 3  ; true"
    );

    define_native!(Lt, env, inter);
    define_help!(
        Lt,
        env,
        "[number number]: returns true if first number is less than second\n\n< 3 5  ; true"
    );

    define_native!(LtE, env, inter);
    define_help!(
        LtE,
        env,
        "[number number]: returns true if first number is less than second or equal\n\n<= 3 3  ; true"
    );

    define_native!(Not, env, inter);
    define_help!(Not, env, "[bool]: logical negation\n\nnot true  ; false");

    define_native!(Or, env, inter);
    define_help!(
        Or,
        env,
        "[bool bool]: logical OR of two boolean values\n\nor true false  ; true"
    );

    define_native!(And, env, inter);
    define_help!(
        And,
        env,
        "[bool bool]: logical AND of two boolean values\n\nand true false  ; false"
    );

    define_native!(Bool, env, inter);
    define_help!(Bool, env, "[value]: converts value to boolean. 0, null, empty string/list/object are false, everything else is true\n\nbool 0  ; false\nbool \"hello\"  ; true");
}
