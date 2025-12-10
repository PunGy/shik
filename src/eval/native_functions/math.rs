use crate::{
    count_args,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeContext, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
    define_native,
};
use std::rc::Rc;

native_op!(Plus, "+", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x + y))
});

native_op!(Minus, "-", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x - y))
});

native_op!(Multiply, "*", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x * y))
});

native_op!(Divide, "/", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x / y))
});

native_op!(Abs, "math.abs", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.abs()))
});

native_op!(Floor, "math.floor", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.floor()))
});

native_op!(Ceil, "math.ceil", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.ceil()))
});

native_op!(Round, "math.round", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.round()))
});

native_op!(Min, "math.min", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x.min(y)))
});

native_op!(Max, "math.max", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x.max(y)))
});

native_op!(Mod, "mod", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x % y))
});

native_op!(Pow, "math.pow", [base, exp], {
    let base = base.expect_number()?;
    let exp = exp.expect_number()?;
    native_result(Value::Number(base.powf(exp)))
});

native_op!(Sqrt, "math.sqrt", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.sqrt()))
});

native_op!(Sin, "math.sin", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.sin()))
});

native_op!(Cos, "math.cos", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.cos()))
});

native_op!(Tan, "math.tan", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.tan()))
});

native_op!(Log, "math.log", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.ln()))
});

native_op!(Log10, "math.log10", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.log10()))
});

pub fn bind_math_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Plus, env, inter);
    define_native!(Minus, env, inter);
    define_native!(Divide, env, inter);
    define_native!(Multiply, env, inter);
    define_native!(Abs, env, inter);
    define_native!(Floor, env, inter);
    define_native!(Ceil, env, inter);
    define_native!(Round, env, inter);
    define_native!(Min, env, inter);
    define_native!(Max, env, inter);
    define_native!(Mod, env, inter);
    define_native!(Pow, env, inter);
    define_native!(Sqrt, env, inter);
    define_native!(Sin, env, inter);
    define_native!(Cos, env, inter);
    define_native!(Tan, env, inter);
    define_native!(Log, env, inter);
    define_native!(Log10, env, inter);
}
