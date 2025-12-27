/// Tree-walk interpretator
use crate::{
    eval::{
        error::RuntimeError,
        native_functions::{
            bool::bind_bool_module, branching::bind_special_module, file::bind_file_module,
            function::bind_function_module, keywords::bind_keywords_module, list::bind_list_module,
            misc::bind_misc_module, number::bind_number_module, polymorphic::bind_poly_module,
            print::bind_print_module, shell::bind_shell_module, string::bind_string_module,
            variables::bind_variable_module,
        },
        value::{
            Closure, Env, EnvRef, NativeClosure, SpecialBoundClosure, SpecialClosure, Value,
            ValueRef,
        },
        EvalResult,
    },
    parser::{Expression, LetPattern, Program},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct InterpretatorContext {
    pub env: EnvRef,
    pub quoted: bool,
}

#[derive(Debug)]
pub struct Interpretator {
    // global context
    pub ctx: RefCell<InterpretatorContext>,
}

impl Interpretator {
    pub fn new() -> Rc<Self> {
        let env = Rc::new(Env::new(None));
        let ctx = RefCell::new(InterpretatorContext {
            env: Rc::clone(&env),
            quoted: false,
        });

        // Create interpretator with the environment
        let inter = Self { ctx };
        let inter = Rc::new(inter);

        // Bind all modules with access to interpretator
        bind_number_module(&env, Rc::clone(&inter));
        bind_bool_module(&env, Rc::clone(&inter));
        bind_string_module(&env, Rc::clone(&inter));
        bind_list_module(&env, Rc::clone(&inter));
        bind_poly_module(&env, Rc::clone(&inter));
        bind_print_module(&env, Rc::clone(&inter));
        bind_keywords_module(&env, Rc::clone(&inter));
        bind_file_module(&env, Rc::clone(&inter));
        bind_shell_module(&env, Rc::clone(&inter));
        bind_variable_module(&env, Rc::clone(&inter));
        bind_special_module(&env, Rc::clone(&inter));
        bind_misc_module(&env, Rc::clone(&inter));
        bind_function_module(&env, Rc::clone(&inter));

        inter
    }

    pub fn interpretate(&self, program: &Program) -> EvalResult {
        let mut last = Rc::new(Value::Null);

        let env = Rc::clone(&self.ctx.borrow().env);
        for stmt in &program.statements {
            last = self.expand(self.eval_expr(&stmt.expression, &env)?)?;
        }

        Ok(last)
    }

    pub fn eval_expr(&self, expr: &Expression, env: &EnvRef) -> EvalResult {
        // println!("---");
        // println!("eval expr: {:?}", expr);
        // println!("With env: {:?}", ctx);
        match expr {
            Expression::Number(x) => Ok(Rc::new(Value::Number(*x))),
            Expression::String(s) => Ok(Rc::new(Value::String(s.clone()))),
            Expression::StringInterpolation(si) => {
                let mut str = si.string.clone();
                let entries = &si.entries;

                for inter in entries.into_iter().rev() {
                    let i = inter.position;
                    let val = self.expand(self.eval_expr(&inter.expression, &env)?)?;
                    let val_str = match val.as_ref() {
                        Value::String(s) => s,
                        val => &val.to_string(),
                    };
                    str.replace_range(i..i + 1, val_str);
                }

                return Ok(Rc::new(Value::String(str)));
            }
            Expression::List(lst) => {
                let mut res: Vec<ValueRef> = Vec::new();

                for it in lst.into_iter() {
                    let val = self.expand(self.eval_expr(it, env)?)?;
                    res.push(val);
                }

                Ok(Rc::new(Value::List(res)))
            }
            Expression::Object(obj) => {
                let mut res: HashMap<String, ValueRef> = HashMap::new();

                for it in obj.iter() {
                    let key = self.expand(self.eval_expr(&it.key, env)?)?;
                    let key = key.expect_string()?;
                    let val = self.expand(self.eval_expr(&it.value, env)?)?;
                    res.insert(key.to_string(), val);
                }

                Ok(Rc::new(Value::Object(res)))
            }
            Expression::Pipe { left, right } => {
                let f = self.eval_expr(right.as_ref(), env)?;

                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let mut curried = SpecialClosure::new(
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.interpretator),
                            Rc::clone(&env),
                        );
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*left.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    _ => {
                        let a = self.expand(self.eval_expr(left.as_ref(), env)?)?;

                        self.apply_fn(&f, &a)
                    }
                }
            }
            Expression::Chain { left, right } => {
                let f = self.eval_expr(left.as_ref(), env)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let mut curried = SpecialClosure::new(
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.interpretator),
                            Rc::clone(&env),
                        );
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*right.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    _ => {
                        let a = self.expand(self.eval_expr(right.as_ref(), env)?)?;

                        self.apply_fn(&f, &a)
                    }
                }
            }
            Expression::Application { function, argument } => {
                let f = self.eval_expr(function.as_ref(), env)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let mut curried = SpecialClosure::new(
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.interpretator),
                            Rc::clone(&env),
                        );
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*argument.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    Value::SpecialBoundForm(closure) => {
                        if closure.params_count == 0 {
                            return closure.exec();
                        }

                        // Make a new curried lambda
                        let mut curried = SpecialBoundClosure::new(
                            closure.params_count,
                            Rc::clone(&closure.logic),
                            Rc::clone(&closure.inter),
                            Rc::clone(&closure.env),
                        );
                        curried.binded.extend_from_slice(&closure.binded);
                        curried.binded.push(*argument.clone());

                        if curried.binded.len() == closure.params_count {
                            curried.exec()
                        } else {
                            Ok(Rc::new(Value::SpecialBoundForm(curried)))
                        }
                    }
                    _ => {
                        let a = self.expand(self.eval_expr(argument.as_ref(), env)?)?;

                        self.apply_fn(&f, &a)
                    }
                }
            }
            Expression::Parenthesized(expr) => self.eval_expr(expr, env),
            Expression::Block(expr_lst) => {
                let mut last = Rc::new(Value::Null);

                for it in expr_lst.iter() {
                    last = self.expand(self.eval_expr(it, env)?)?;
                }

                Ok(last)
            }
            Expression::Lambda {
                parameters,
                #[allow(unused_variables)] // rest still not supported
                rest,
                body,
            } => Ok(Rc::new(Value::Lambda(Closure::new(
                parameters.clone(),
                body.clone(),
                Rc::new(Env::new(Some(Rc::clone(env)))),
            )))),
            Expression::Let { pattern, value } => match pattern {
                LetPattern::Identifier(name) => {
                    let val = self.expand(self.eval_expr(value, env)?)?;
                    env.define(name.to_string(), Rc::clone(&val));
                    Ok(val)
                }
                _ => Err(RuntimeError::NotYetImplemented(expr.clone())),
            },
            Expression::Identifier(name) => {
                env.lookup(name)
                    .map_or(
                        Err(RuntimeError::UndefinedVariable(name.clone())),
                        |val| match val.as_ref() {
                            Value::Lambda(closure) => {
                                let quoted = self.ctx.borrow().quoted;
                                if closure.params.len() == 0 && !quoted {
                                    return self.eval_expr(&closure.body, &closure.env);
                                }
                                Ok(val)
                            }
                            Value::NativeLambda(closure) => {
                                if closure.params_count == 0 {
                                    return closure.exec();
                                }
                                Ok(val)
                            }
                            _ => Ok(val),
                        },
                    )
            }
            e => Err(RuntimeError::NotYetImplemented(e.clone())),
        }
    }

    pub fn apply_fn(&self, f: &ValueRef, a: &ValueRef) -> EvalResult {
        match f.as_ref() {
            Value::Lambda(closure) => {
                if closure.params.len() == 0 {
                    return self.eval_expr(&closure.body, &closure.env);
                }

                let mut curried = closure.clone();
                curried.binded.push(a.clone());

                if curried.binded.len() == curried.params.len() {
                    // All params are binded, let's evaluate
                    curried.bind_variables();
                    // println!("<--apply body");

                    self.eval_expr(&curried.body, &curried.env)
                } else {
                    // Make a new curried lambda
                    Ok(Rc::new(Value::Lambda(curried)))
                }
            }
            Value::NativeLambda(closure) => {
                if closure.params_count == 0 {
                    return closure.exec();
                }

                // Make a new curried lambda
                let mut curried = NativeClosure::new(
                    closure.params_count,
                    Rc::clone(&closure.logic),
                    Rc::clone(&closure.inter),
                    Rc::clone(&closure.env),
                );
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
