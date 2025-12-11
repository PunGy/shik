use crate::{
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        value::{EnvRef, SpecialClosure, SpecialFn, Value, NativeContext},
        EvalResult,
    },
    parser::Expression, special_op,
};
use std::{rc::Rc};

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

special_op!(If, "if", args, ctx, {
        let args_count = args.len();
        if args_count < 2 {
            return Err(RuntimeError::InvalidApplication);
        }

        let mut args_it = args.iter().peekable();
        let predicate = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
        let mut predicate = ctx.inter.eval_expr(predicate, &ctx.env)?.expect_bool()?;

        if args_count == 2 {
            // Simple if without else
            if predicate {
                ctx.inter.eval_expr(&args_it.next().unwrap(), &ctx.env)
            } else {
                Ok(Rc::new(Value::Null))
            }
        } else if args_count % 2 == 0 {
            // Without else at the end
            while !predicate {
                args_it.next(); // skip body
                let next = args_it.next();
                if next == None {
                    break;
                }
                predicate = ctx.inter.eval_expr(next.unwrap(), &ctx.env)?.expect_bool()?;
            }

            if predicate {
                let next = args_it.next().ok_or(RuntimeError::InvalidApplication)?;
                ctx.inter.eval_expr(next, &ctx.env)
            } else {
                Ok(Rc::new(Value::Null))
            }
        } else {
            // With else at the end
            let mut next: Option<&Expression> = args_it.next();

            while !predicate {
                next = args_it.next();
                // if it is the last - go back, we found final `else`
                if args_it.peek() == None {
                    break;
                }
                predicate = ctx.inter.eval_expr(next.unwrap(), &ctx.env)?.expect_bool()?;
                if predicate {
                    // next body
                    next = args_it.next();
                    break
                } else {
                    args_it.next();
                }
            }

            // the next would be the desired body for sure, either `elseif` block, or `else`
            let next = next.ok_or(RuntimeError::InvalidApplication)?;
            ctx.inter.eval_expr(next, &ctx.env)
        }
});

pub fn bind_special_module(env: &EnvRef, interpretator: Rc<Interpretator>) {
    If::define(env, interpretator);
}
