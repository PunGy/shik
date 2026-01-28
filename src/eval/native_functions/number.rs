use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{
            EnvRef, NativeClosure, NativeContext, NativeFn, SpecialClosure, SpecialFn, Value,
            ValueRef,
        },
        EvalResult,
    },
    native_op,
    parser::Expression,
    special_op,
};
use rand::prelude::*;
use std::rc::Rc;

native_op!(Plus, "number.+", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x + y))
});

native_op!(Minus, ["-", "number.-"], [y, x], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x - y))
});

native_op!(Multiply, ["*", "number.*"], [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x * y))
});

native_op!(Divide, ["/", "number./"], [y, x], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;

    native_result(Value::Number(x / y))
});

native_op!(Mod, ["%", "number.%"], [y, x], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x % y))
});

native_op!(Abs, "number.abs", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.abs()))
});

native_op!(Floor, "number.floor", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.floor()))
});

native_op!(Ceil, "number.ceil", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.ceil()))
});

native_op!(Round, "number.round", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.round()))
});

native_op!(Min, "number.min", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x.min(y)))
});

native_op!(Max, "number.max", [x, y], {
    let x = x.expect_number()?;
    let y = y.expect_number()?;
    native_result(Value::Number(x.max(y)))
});

native_op!(Pow, ["^", "number.pow"], [exp, base], {
    let base = base.expect_number()?;
    let exp = exp.expect_number()?;
    native_result(Value::Number(base.powf(exp)))
});

native_op!(Sqrt, "number.sqrt", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.sqrt()))
});

native_op!(Sin, "number.sin", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.sin()))
});

native_op!(Cos, "number.cos", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.cos()))
});

native_op!(Tan, "number.tan", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.tan()))
});

native_op!(Log, "number.log", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.ln()))
});

native_op!(Log10, "number.log10", [x], {
    let x = x.expect_number()?;
    native_result(Value::Number(x.log10()))
});

// Random number generator with variable arguments:
// - 0 args: random float 0.0 to 1.0
// - 1 arg (max): random integer 0 to max (exclusive)
// - 2 args (min, max): random integer min to max (exclusive)
special_op!(RandNumber, "number.rand", args, ctx, {
    let mut rng = rand::rng();

    match args.len() {
        0 => {
            // No arguments: return random float 0.0 to 1.0
            let n = rng.random::<f64>();
            native_result(Value::Number(n))
        }
        1 => {
            // One argument: return random integer 0 to max (exclusive)
            let max = ctx.inter.eval_expand(&args[0], &ctx.env)?.expect_number()? as i64;
            if max <= 0 {
                return native_result(Value::Number(0.0));
            }
            let n = rng.random_range(0..max);
            native_result(Value::Number(n as f64))
        }
        2 => {
            // Two arguments: return random integer min to max (exclusive)
            let min = ctx.inter.eval_expand(&args[0], &ctx.env)?.expect_number()? as i64;
            let max = ctx.inter.eval_expand(&args[1], &ctx.env)?.expect_number()? as i64;
            if max <= min {
                return native_result(Value::Number(min as f64));
            }
            let n = rng.random_range(min..max);
            native_result(Value::Number(n as f64))
        }
        count => Err(RuntimeError::invalid_application(format!("(number.rand) wrong number of arguments. Must be 1, 2 or 3. Got {}", count))),
    }
});

pub fn bind_number_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "number.".to_string(),
        "number module:

Arithmetic:
- number.+: addition
- -, number.-: subtraction
- *, number.*: multiplication
- /, number./: division
- %, number.%: modulo (remainder)
- ^, number.pow: exponentiation

Rounding:
- number.abs: absolute value
- number.floor: rounds down
- number.ceil: rounds up
- number.round: rounds to nearest

Comparison:
- number.min: smaller of two
- number.max: larger of two

Math functions:
- number.sqrt: square root
- number.sin: sine (radians)
- number.cos: cosine (radians)
- number.tan: tangent (radians)
- number.log: natural logarithm
- number.log10: base-10 logarithm

Random:
- number.rand: random number (variadic)"
            .to_string(),
    );

    define_native!(Plus, env, inter);
    define_help!(
        Plus,
        env,
        "[number number]: adds two numbers\n\nnumber.+ 2 3  ; 5"
    );

    define_native!(Minus, env, inter);
    define_help!(
        Minus,
        env,
        "[number number]: subtracts second from first\n\n5 $> - 3  ; 2"
    );

    define_native!(Divide, env, inter);
    define_help!(
        Divide,
        env,
        "[number number]: divides first by second\n\n10 $> / 2  ; 5"
    );

    define_native!(Multiply, env, inter);
    define_help!(
        Multiply,
        env,
        "[number number]: multiplies two numbers\n\n* 3 4  ; 12"
    );

    define_native!(Abs, env, inter);
    define_help!(
        Abs,
        env,
        "[number]: returns absolute value\n\nnumber.abs -5  ; 5"
    );

    define_native!(Floor, env, inter);
    define_help!(
        Floor,
        env,
        "[number]: rounds down to nearest integer\n\nnumber.floor 3.7  ; 3"
    );

    define_native!(Ceil, env, inter);
    define_help!(
        Ceil,
        env,
        "[number]: rounds up to nearest integer\n\nnumber.ceil 3.2  ; 4"
    );

    define_native!(Round, env, inter);
    define_help!(
        Round,
        env,
        "[number]: rounds to nearest integer\n\nnumber.round 3.5  ; 4"
    );

    define_native!(Min, env, inter);
    define_help!(
        Min,
        env,
        "[number number]: returns smaller of two numbers\n\nnumber.min 3 5  ; 3"
    );

    define_native!(Max, env, inter);
    define_help!(
        Max,
        env,
        "[number number]: returns larger of two numbers\n\nnumber.max 3 5  ; 5"
    );

    define_native!(Mod, env, inter);
    define_help!(
        Mod,
        env,
        "[number number]: returns remainder of division\n\n10 $> % 3  ; 1"
    );

    define_native!(Pow, env, inter);
    define_help!(
        Pow,
        env,
        "[exp:number base:number]: raises base to exponent power\n\n2 $> ^ 3  ; 8"
    );

    define_native!(Sqrt, env, inter);
    define_help!(
        Sqrt,
        env,
        "[number]: returns square root\n\nnumber.sqrt 16  ; 4"
    );

    define_native!(Sin, env, inter);
    define_help!(
        Sin,
        env,
        "[number]: returns sine (radians)\n\nnumber.sin 0  ; 0"
    );

    define_native!(Cos, env, inter);
    define_help!(
        Cos,
        env,
        "[number]: returns cosine (radians)\n\nnumber.cos 0  ; 1"
    );

    define_native!(Tan, env, inter);
    define_help!(
        Tan,
        env,
        "[number]: returns tangent (radians)\n\nnumber.tan 0  ; 0"
    );

    define_native!(Log, env, inter);
    define_help!(
        Log,
        env,
        "[number]: returns natural logarithm (ln)\n\nnumber.log 2.718  ; ~1"
    );

    define_native!(Log10, env, inter);
    define_help!(
        Log10,
        env,
        "[number]: returns base-10 logarithm\n\nnumber.log10 100  ; 2"
    );

    RandNumber::define(&env, Rc::clone(&inter));
    define_help!(RandNumber, env, "[] or [max:number] or [min:number max:number]: generates random numbers\n\nnumber.rand  ; random float 0.0 to 1.0\nnumber.rand 10  ; random integer 0 to 9\nnumber.rand 5 10  ; random integer 5 to 9");
}
