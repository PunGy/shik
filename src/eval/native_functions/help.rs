use crate::{
    define_help, define_native,
    eval::{
        evaluator::Interpretator,
        native_functions::{native_result, polymorphic::Print},
        value::{EnvRef, NativeContext, SpecialClosure, SpecialFn, Value},
        EvalResult,
    },
    parser::Expression,
    special_op,
};
use std::rc::Rc;

const GENERAL_HELP: &str = "SHIK language:

To get help on any module of the language: help [module_name]

Example:

```shik
> help number.

number module:

Arithmetic:
- number.+: addition
...
```

MODULES:


-- Type modules
- number.: arithmetic, rounding, comparison, math functions, random
- string.: string manipulation, conversion, iteration
- list.: list operations, higher-order functions
- bool.: comparison, logic, conversion

-- Function modules
- fn.: function utilities (invoke, quote)

-- System modules
- file.: file reading, writing, operations, information
- path.: path manipulation (name, stem, ext, parent, join, absolute)
- shell.: shell execution, environment variables, I/O
- process.: process control (pid, args, sleep, exit, abort)

-- Language features
- branching.: loops, conditionals (if, while, match)
- var.: variable operations (get by name)

-- Misc
- general.: polymorphic operations for any value (+, at, iterate)
- misc.: utility functions (or?)";

special_op!(Help, "help", args, ctx, {
    let msg: String;
    if args.len() > 0 {
        let arg = &args[0];

        match arg {
            Expression::Identifier(ident) => {
                let help_msg = ctx.env.lookup_help(ident);
                match help_msg {
                    Some(help_msg) => msg = help_msg,
                    None => {
                        msg = help_for_literal(&arg, ctx, Some(format!("Identifier {}\n", ident)))
                    }
                }
            }
            _ => msg = help_for_literal(&arg, ctx, None),
        }
    } else {
        msg = GENERAL_HELP.to_string();
    }

    let value = Rc::new(Value::String(msg));

    Print::run(&value)?;

    native_result(Value::Null)
});

fn help_for_literal(expr: &Expression, ctx: &NativeContext, prefix: Option<String>) -> String {
    let val = ctx.inter.eval_expr(expr, &ctx.env);
    match val {
        Ok(val) => format!(
            "{}{:?}: {}",
            prefix.unwrap_or("".to_string()),
            val.get_type(),
            val.to_string()
        ),
        Err(_) => "Unable to get help".to_string(),
    }
}

pub fn bind_help_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Help, env, inter);
    define_help!(Help, env, "[value?]: print help information about anything. Enter help without arguments to get general information about the language.");
}
