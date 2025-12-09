use crate::{
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, SpecialClosure, SpecialFn, Value, ValueRef},
        EvalResult,
    },
    parser::Expression,
};
use std::{os::unix::process::parent_id, rc::Rc};

/*
; count 2
if predicate
    1 ; if true

; count 3
if predicate
    1 ; if true
    2 ; else

; count 4
if predicate
    1 ; if true
   predicate ; else if
    2 ; else if true

; count 5
if predicate
    1 ; if true
   predicate ; else if
    2 ; else if true
    3 ; else
*/

#[derive(Debug)]
pub struct If;

impl SpecialFn for If {
    fn exec(&self, args: &Vec<Expression>, inter: &Interpretator) -> EvalResult {
        let args_count = args.len();
        if args_count < 2 {
            return Err(RuntimeError::InvalidApplication);
        }

        let mut args_it = args.iter().peekable();
        let predicate = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
        let mut predicate = inter.eval_expr(predicate, &inter.ctx)?.expect_bool()?;

        if args_count == 2 {
            // Simple if without else
            if predicate {
                inter.eval_expr(&args_it.next().unwrap(), &inter.ctx)
            } else {
                Ok(Rc::new(Value::Bool(false)))
            }
        } else if args_count % 2 == 0 {
            // Without else at the end
            while !predicate {
                args_it.next(); // skip body
                let next = args_it.next();
                if next == None {
                    break;
                }
                predicate = inter.eval_expr(next.unwrap(), &inter.ctx)?.expect_bool()?;
            }

            if predicate {
                let next = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
                inter.eval_expr(next, &inter.ctx)
            } else {
                Ok(Rc::new(Value::Bool(false)))
            }
        } else {
            let mut next: Option<&Expression> = None;
            // With else at the end
            while !predicate {
                args_it.next(); // skip body
                next = args_it.next();
                // if it is the last - go back, we found final `else`
                if args_it.peek() == None {
                    break;
                }
                predicate = inter.eval_expr(next.unwrap(), &inter.ctx)?.expect_bool()?;
                if predicate {
                    // next body
                    next = args_it.next();
                    break
                }
            }

            // the next would be the desired body for sure, either `elseif` block, or `else`
            let next = next.ok_or(RuntimeError::InvalidApplication)?;
            inter.eval_expr(next, &inter.ctx)
        }
    }
}

impl If {
    pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
        env.define(
            "if".to_string(),
            Rc::new(Value::SpecialForm(SpecialClosure::new(Rc::new(If), inter))),
        );
    }
}

pub fn bind_special_module(env: &EnvRef, interpretator: Rc<Interpretator>) {
    If::define(env, interpretator);
}
