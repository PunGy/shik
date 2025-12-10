use crate::{
    define_native, eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, SpecialClosure, SpecialFn, Value},
        EvalResult,
    }, parser::Expression
};
use std::{rc::Rc};

#[derive(Debug)]
pub struct Call;

impl SpecialFn for Call {
    fn exec(&self, args: &Vec<Expression>, inter: &Interpretator, env: &EnvRef) -> EvalResult {
        let mut args_it = args.iter().peekable();
        let fun = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
        let mut fun = inter.eval_expr(fun, &env)?;

        if args.len() == 1 {
            // If called without arguments - pass null
            return inter.apply_fn(&fun, &Rc::new(Value::Null));
        }

        let mut arg = args_it.next();
        while arg != None {
            let arg_val = inter.eval_expr(arg.unwrap(), &env)?;
            fun = inter.apply_fn(&fun, &arg_val)?;
            arg = args_it.next();
        }
        Ok(fun)
    }
}

impl Call {
    pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
        let ctx = Rc::clone(&env);
        env.define(
            "call".to_string(),
            Rc::new(Value::SpecialForm(SpecialClosure::new(Rc::new(Call), inter, ctx))),
        );
    }
}

pub fn bind_keywords_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Yes, true and false are just variables, you can override them :D
    env.define("true".to_string(), Rc::new(Value::Bool(true)));
    env.define("false".to_string(), Rc::new(Value::Bool(false)));

    env.define("null".to_string(), Rc::new(Value::Null));

    define_native!(Call, env, inter);
}
