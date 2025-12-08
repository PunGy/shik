/// Tree-walk interpretator
use crate::{
    eval::{
        error::RuntimeError,
        native_functions::math::bind_math_module,
        value::{Env, EnvRef, NativeClosure, Value, ValueRef},
        EvalResult,
    },
    parser::{Expression, Program},
};
use std::rc::Rc;

#[derive(Clone)]
struct EvalContext {
    pub env: EnvRef,
}

impl EvalContext {
    pub fn new_global() -> Self {
        EvalContext {
            env: Rc::new(Env::new(None)),
        }
    }
}

pub fn interpretate(program: &Program) -> EvalResult {
    let mut ctx = EvalContext::new_global();
    bind_math_module(&ctx.env);

    let mut last = Rc::new(Value::Null);

    for stmt in &program.statements {
        last = eval_expr(&stmt.expression, &mut ctx)?;
    }

    Ok(last)
}

fn eval_expr(expr: &Expression, ctx: &mut EvalContext) -> EvalResult {
    // print!("eval expr: {:?}", expr);
    match expr {
        Expression::Number(x) => Ok(Rc::new(Value::Number(*x))),
        Expression::Application { function, argument } => {
            let f = eval_expr(function.as_ref(), ctx)?;
            let a = eval_expr(argument.as_ref(), ctx)?;

            apply_fn(&f, &a, &ctx)
        }
        Expression::Identifier(name) => ctx
            .env
            .lookup(name)
            .map_or(Err(RuntimeError::UndefinedVariable(name.clone())), Ok),
        e => Err(RuntimeError::NotYetImplemented(e.clone())),
    }
}

fn apply_fn(f: &ValueRef, a: &ValueRef, ctx: &EvalContext) -> EvalResult {
    match f.as_ref() {
        Value::Lambda(closure) => {
            let closure = closure.as_ref();
            if closure.binded.len() == closure.params.len() {
                // All params are binded, let's evaluate
                let mut fn_context = ctx.clone();
                closure.bind_variables(Rc::clone(&fn_context.env));

                eval_expr(&closure.body, &mut fn_context)
            } else {
                // Make a new curried lambda
                let mut curried = closure.clone();
                curried.binded.push(a.clone());
                Ok(Rc::new(Value::Lambda(Rc::new(curried))))
            }
        }
        Value::NativeLambda(closure) => {
            let closure = closure.as_ref();
            // Make a new curried lambda
            let mut curried = NativeClosure::new(closure.params_count, Rc::clone(&closure.logic));
            curried.binded.extend_from_slice(&closure.binded);
            curried.binded.push(a.clone());

            if curried.binded.len() == closure.params_count {
                curried.exec()
            } else {
                Ok(Rc::new(Value::NativeLambda(Rc::new(curried))))
            }
        }
        _ => Ok(f.clone()),
    }
}
