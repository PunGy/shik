/// Tree-walk interpretator
use crate::{
    eval::{
        error::RuntimeError,
        native_functions::math::bind_math_module,
        value::{Closure, Env, EnvRef, NativeClosure, Value, ValueRef},
        EvalResult,
    },
    parser::{Expression, LetPattern, Program},
};
use std::rc::Rc;

pub struct Interpretator {
    // global context
    ctx: EnvRef,
}

impl Interpretator {
    pub fn new() -> Self {
        Self {
            ctx: Interpretator::new_global()
        }
    }

    pub fn interpretate(&self, program: &Program) -> EvalResult {
        let mut last = Rc::new(Value::Null);

        for stmt in &program.statements {
            last = self.eval_expr(&stmt.expression, &self.ctx)?;
        }

        Ok(last)
    }

    fn new_global() -> EnvRef {
        let env = Rc::new(Env::new(None));

        bind_math_module(&env);

        env
    }

    fn eval_expr(&self, expr: &Expression, ctx: &EnvRef) -> EvalResult {
        // println!("---");
        // println!("eval expr: {:?}", expr);
        // println!("With env: {:?}", ctx);
        match expr {
            Expression::Number(x) => Ok(Rc::new(Value::Number(*x))),
            Expression::Application { function, argument } => {
                let f = self.eval_expr(function.as_ref(), ctx)?;
                let a = self.eval_expr(argument.as_ref(), ctx)?;

                self.apply_fn(&f, &a)
            }
            Expression::Parenthesized(expr) => self.eval_expr(expr, ctx),
            Expression::Lambda {
                parameters,
                rest,
                body,
            } => Ok(Rc::new(Value::Lambda(Rc::new(Closure::new(
                parameters.clone(),
                body.clone(),
                Rc::new(Env::new(Some(Rc::clone(ctx)))),
            ))))),
            Expression::Let { pattern, value } => match pattern {
                LetPattern::Identifier(name) => {
                    let val = self.eval_expr(value, ctx)?;
                    ctx.define(name.to_string(), Rc::clone(&val));
                    Ok(val)
                }
                _ => Err(RuntimeError::NotYetImplemented(expr.clone())),
            },
            Expression::Identifier(name) => ctx
                .lookup(name)
                .map_or(Err(RuntimeError::UndefinedVariable(name.clone())), Ok),
            e => Err(RuntimeError::NotYetImplemented(e.clone())),
        }
    }

    fn apply_fn(&self, f: &ValueRef, a: &ValueRef) -> EvalResult {
        match f.as_ref() {
            Value::Lambda(closure) => {
                let mut curried = closure.as_ref().clone();
                curried.binded.push(a.clone());

                if curried.binded.len() == curried.params.len() {
                    // All params are binded, let's evaluate
                    curried.bind_variables();
                    // println!("<--apply body");

                    self.eval_expr(&closure.body, &closure.env)
                } else {
                    // Make a new curried lambda
                    Ok(Rc::new(Value::Lambda(Rc::new(curried))))
                }
            }
            Value::NativeLambda(closure) => {
                let closure = closure.as_ref();
                // Make a new curried lambda
                let mut curried =
                    NativeClosure::new(closure.params_count, Rc::clone(&closure.logic));
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
}
