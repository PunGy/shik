/// Tree-walk interpretator
use crate::{
    eval::{
        error::RuntimeError,
        native_functions::{
            bool::bind_bool_module, branching::bind_special_module, file::bind_file_module, keywords::bind_keywords_module, list::bind_list_module, math::bind_math_module, print::bind_print_module, string::bind_string_module
        },
        value::{Closure, Env, EnvRef, NativeClosure, SpecialClosure, Value, ValueRef},
        EvalResult,
    },
    parser::{Expression, LetPattern, Program},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Interpretator {
    // global context
    pub ctx: EnvRef,
}

impl Interpretator {
    pub fn new() -> Rc<Self> {
        let inter = Self {
            ctx: Interpretator::new_global(),
        };
        let inter = Rc::new(inter);

        bind_special_module(&inter.ctx, Rc::clone(&inter));

        inter
    }

    pub fn interpretate(&self, program: &Program) -> EvalResult {
        let mut last = Rc::new(Value::Null);

        for stmt in &program.statements {
            last = self.expand(self.eval_expr(&stmt.expression, &self.ctx)?)?;
        }

        Ok(last)
    }

    fn new_global() -> EnvRef {
        let env = Rc::new(Env::new(None));

        bind_math_module(&env);
        bind_bool_module(&env);
        bind_string_module(&env);
        bind_list_module(&env);
        bind_print_module(&env);
        bind_keywords_module(&env);
        bind_file_module(&env);

        env
    }

    pub fn eval_expr(&self, expr: &Expression, ctx: &EnvRef) -> EvalResult {
        // println!("---");
        // println!("eval expr: {:?}", expr);
        // println!("With env: {:?}", ctx);
        match expr {
            Expression::Number(x) => Ok(Rc::new(Value::Number(*x))),
            Expression::String(s) => Ok(Rc::new(Value::String(s.clone()))),
            Expression::List(lst) => {
                let mut res: Vec<ValueRef> = Vec::new();

                for it in lst.into_iter() {
                    let val = self.expand(self.eval_expr(it, ctx)?)?;
                    res.push(val);
                }

                Ok(Rc::new(Value::List(res)))
            }
            Expression::Object(obj) => {
                let mut res: HashMap<String, ValueRef> = HashMap::new();

                for it in obj.iter() {
                    let key = self.expand(self.eval_expr(&it.key, ctx)?)?;
                    let key = key.expect_string()?;
                    let val = self.expand(self.eval_expr(&it.value, ctx)?)?;
                    res.insert(key.to_string(), val);
                }

                Ok(Rc::new(Value::Object(res)))
            }
            Expression::Pipe { left, right } => {
                let a = self.expand(self.eval_expr(left.as_ref(), ctx)?)?;
                let f = self.eval_expr(right.as_ref(), ctx)?;

                self.apply_fn(&f, &a)
            }
            Expression::Chain { left, right } => {
                let f = self.eval_expr(left.as_ref(), ctx)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let mut curried = SpecialClosure::new(
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.interpretator),
                            Rc::clone(&ctx),
                        );
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*right.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    _ => {
                        let a = self.expand(self.eval_expr(right.as_ref(), ctx)?)?;

                        self.apply_fn(&f, &a)
                    }
                }
            }
            Expression::Application { function, argument } => {
                let f = self.eval_expr(function.as_ref(), ctx)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let mut curried = SpecialClosure::new(
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.interpretator),
                            Rc::clone(&ctx),
                        );
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*argument.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    _ => {
                        let a = self.expand(self.eval_expr(argument.as_ref(), ctx)?)?;

                        self.apply_fn(&f, &a)
                    }
                }
            }
            Expression::Parenthesized(expr) => self.eval_expr(expr, ctx),
            Expression::Block(expr_lst) => {
                let mut last = Rc::new(Value::Null);

                for it in expr_lst.iter() {
                    last = self.expand(self.eval_expr(it, ctx)?)?;
                }

                Ok(last)
            }
            Expression::Lambda {
                parameters,
                rest,
                body,
            } => Ok(Rc::new(Value::Lambda(Closure::new(
                parameters.clone(),
                body.clone(),
                Rc::new(Env::new(Some(Rc::clone(ctx)))),
            )))),
            Expression::Let { pattern, value } => match pattern {
                LetPattern::Identifier(name) => {
                    let val = self.expand(self.eval_expr(value, ctx)?)?;
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
                let mut curried = closure.clone();
                curried.binded.push(a.clone());

                if curried.binded.len() == curried.params.len() {
                    // All params are binded, let's evaluate
                    curried.bind_variables();
                    // println!("<--apply body");

                    self.eval_expr(&closure.body, &closure.env)
                } else {
                    // Make a new curried lambda
                    Ok(Rc::new(Value::Lambda(curried)))
                }
            }
            Value::NativeLambda(closure) => {
                // Make a new curried lambda
                let mut curried =
                    NativeClosure::new(closure.params_count, Rc::clone(&closure.logic));
                curried.binded.extend_from_slice(&closure.binded);
                curried.binded.push(a.clone());

                if curried.binded.len() == closure.params_count {
                    curried.exec()
                } else {
                    Ok(Rc::new(Value::NativeLambda(curried)))
                }
            }
            _ => Ok(f.clone()),
        }
    }

    fn expand(&self, v: ValueRef) -> EvalResult {
        match v.as_ref() {
            Value::SpecialForm(closure) => closure.exec(),
            _ => Ok(v),
        }
    }
}
