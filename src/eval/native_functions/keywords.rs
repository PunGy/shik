use crate::{
    define_native, eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, SpecialClosure, SpecialFn, Value, NativeContext},
        EvalResult,
    }, parser::Expression,
    special_op,
};
use std::{rc::Rc};

special_op!(Call, "call", args, ctx, {
        let mut args_it = args.iter().peekable();
        let fun = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
        let mut fun = ctx.inter.eval_expr(fun, &ctx.env)?;

        if args.len() == 1 {
            // If called without arguments - pass null
            return ctx.apply(&fun, &Rc::new(Value::Null));
        }

        let mut arg = args_it.next();
        while arg != None {
            let arg_val = ctx.inter.eval_expr(arg.unwrap(), &ctx.env)?;
            fun = ctx.apply(&fun, &arg_val)?;
            arg = args_it.next();
        }
        Ok(fun)
});

pub fn bind_keywords_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Yes, true and false are just variables, you can override them :D
    env.define("true".to_string(), Rc::new(Value::Bool(true)));
    env.define("false".to_string(), Rc::new(Value::Bool(false)));

    env.define("null".to_string(), Rc::new(Value::Null));

    define_native!(Call, env, inter);
}
