/// Tree-walk interpretator
use crate::{
    eval::{
        error::RuntimeError,
        native_functions::{
            bool::{bind_bool_module, Eq},
            branching::bind_special_module,
            file::bind_file_module,
            function::bind_function_module,
            help::bind_help_module,
            keywords::bind_keywords_module,
            list::bind_list_module,
            misc::bind_misc_module,
            number::bind_number_module,
            object::bind_object_module,
            polymorphic::bind_poly_module,
            shell::bind_shell_module,
            string::bind_string_module,
            variables::bind_variable_module,
        },
        utils::{define_match, pattern_match},
        value::{
            null_value, Closure, Env, EnvRef, ListRepr, MatchContext, NativeClosure,
            SpecialBoundClosure, SpecialClosure, Value, ValueRef,
        },
        EvalResult,
    },
    parser::{Expression, ExpressionKind, MatchPattern, Program, Span},
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
        bind_object_module(&env, Rc::clone(&inter));
        bind_poly_module(&env, Rc::clone(&inter));
        bind_keywords_module(&env, Rc::clone(&inter));
        bind_file_module(&env, Rc::clone(&inter));
        bind_shell_module(&env, Rc::clone(&inter));
        bind_variable_module(&env, Rc::clone(&inter));
        bind_special_module(&env, Rc::clone(&inter));
        bind_misc_module(&env, Rc::clone(&inter));
        bind_function_module(&env, Rc::clone(&inter));
        bind_help_module(&env, Rc::clone(&inter));

        inter
    }

    pub fn interpretate(&self, program: &Program) -> EvalResult {
        let mut last = null_value();

        let env = Rc::clone(&self.ctx.borrow().env);
        for stmt in &program.statements {
            last = self.eval_expand(&stmt.expression, &env)?;
        }

        Ok(last)
    }

    pub fn eval_expr(&self, expr: &Expression, env: &EnvRef) -> EvalResult {
        let span = expr.span;
        match &expr.kind {
            ExpressionKind::Number(x) => Ok(Rc::new(Value::Number(*x))),
            ExpressionKind::String(s) => Ok(Rc::new(Value::String(s.clone()))),
            ExpressionKind::StringInterpolation(si) => {
                let mut str = si.string.clone();
                let entries = &si.entries;

                for inter in entries.into_iter().rev() {
                    let i = inter.position;
                    let val = self.eval_expand(&inter.expression, &env)?;
                    let val_str = match val.as_ref() {
                        Value::String(s) => s,
                        val => &val.to_string(),
                    };
                    str.replace_range(i..i + 1, val_str);
                }

                return Ok(Rc::new(Value::String(str)));
            }
            ExpressionKind::List(lst) => {
                let mut res: Vec<ValueRef> = Vec::with_capacity(lst.len());

                for it in lst.into_iter() {
                    let val = self.eval_expand(it, env)?;
                    res.push(val);
                }

                Ok(Rc::new(Value::List(ListRepr::from_vec(res))))
            }
            ExpressionKind::Object(obj) => {
                let mut res: HashMap<String, ValueRef> = HashMap::new();

                for it in obj.iter() {
                    let key = self.eval_expand(&it.key, env)?;
                    let key = key.expect_string().map_err(|e| e.with_span(it.key.span))?;
                    let val = self.eval_expand(&it.value, env)?;
                    res.insert(key.to_string(), val);
                }

                Ok(Rc::new(Value::Object(res)))
            }
            ExpressionKind::Pipe { left, right } => {
                let f = self.eval_expr(right.as_ref(), env)?;

                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        // Upgrade weak references for the new closure
                        let inter = closure.interpretator.clone();
                        let mut curried = SpecialClosure::new(Rc::clone(&closure.logic), inter);
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*left.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    Value::SpecialBoundForm(closure) => self
                        .apply_special_fn(&closure, &left, env)
                        .map_err(|e| e.with_span(span)),
                    _ => {
                        let a = self.eval_expand(left.as_ref(), env)?;

                        self.apply_fn(&f, &a, env).map_err(|e| e.with_span(span))
                    }
                }
            }
            ExpressionKind::Chain { left, right } => {
                let f = self.eval_expr(left.as_ref(), env)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let inter = closure.interpretator.clone();
                        let mut curried = SpecialClosure::new(Rc::clone(&closure.logic), inter);
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*right.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    Value::SpecialBoundForm(closure) => self
                        .apply_special_fn(&closure, &right, env)
                        .map_err(|e| e.with_span(span)),
                    _ => {
                        let a = self.eval_expand(right.as_ref(), env)?;

                        self.apply_fn(&f, &a, env).map_err(|e| e.with_span(span))
                    }
                }
            }
            ExpressionKind::Application { function, argument } => {
                let f = self.eval_expr(function.as_ref(), env)?;
                match f.as_ref() {
                    Value::SpecialForm(closure) => {
                        let inter = closure.interpretator.clone();
                        let mut curried = SpecialClosure::new(Rc::clone(&closure.logic), inter);
                        curried.params.extend_from_slice(&closure.params);
                        curried.params.push(*argument.clone());
                        let f = Value::SpecialForm(curried);

                        Ok(Rc::new(f))
                    }
                    Value::SpecialBoundForm(closure) => self
                        .apply_special_fn(&closure, &argument, env)
                        .map_err(|e| e.with_span(span)),
                    _ => {
                        let a = self.eval_expand(argument.as_ref(), env)?;

                        self.apply_fn(&f, &a, env).map_err(|e| e.with_span(span))
                    }
                }
            }
            ExpressionKind::Parenthesized(expr) => self.eval_expand(expr, env),
            ExpressionKind::Block(expr_lst) => {
                let mut last = null_value();

                for it in expr_lst.iter() {
                    last = self.eval_expand(it, env)?;
                }

                Ok(last)
            }
            ExpressionKind::Lambda {
                parameters,
                rest,
                body,
            } => Ok(Rc::new(Value::Lambda(Closure::new(
                parameters.clone(),
                rest.clone(),
                body.clone(),
                Rc::clone(env),
            )))),
            ExpressionKind::Let { pattern, value } => {
                let val = self.eval_expand(value, env)?;
                define_match(pattern, &val, &env, &MatchContext::Let).map_err(|e| e.with_span(span))
            }
            ExpressionKind::Match { item, entries } => {
                let item_val = self.eval_expand(item, &env)?;

                for entry in entries {
                    let pattern = &entry.pattern;
                    match pattern {
                        MatchPattern::Identifier(ident) => {
                            let val = self.lookup(&ident, &env).map_err(|e| e.with_span(span))?;
                            if self
                                .val_compare(&item_val, &val)
                                .map_err(|e| e.with_span(span))?
                            {
                                return self.eval_expand(&entry.resolve, &env);
                            }
                        }
                        MatchPattern::NamedWildcard(ident) => {
                            env.define(ident.to_string(), Rc::clone(&item_val));
                            return self.eval_expand(&entry.resolve, &env);
                        }
                        _ => {
                            if pattern_match(pattern, &item_val, &env)
                                .map_err(|e| e.with_span(span))?
                            {
                                return self.eval_expand(&entry.resolve, &env);
                            }
                        }
                    }
                }

                Ok(null_value())
            }
            ExpressionKind::Identifier(name) => {
                self.lookup(name, &env).map_err(|e| e.with_span(span))
            }
            ExpressionKind::Flow { left, right } => {
                let mut params: Vec<MatchPattern> = Vec::new();
                let param = MatchPattern::Identifier("x".to_string());
                params.push(param);

                let body = Expression::new(
                    ExpressionKind::Application {
                        function: right.clone(),
                        argument: Box::new(Expression::new(
                            ExpressionKind::Application {
                                function: left.clone(),
                                argument: Box::new(Expression::new(
                                    ExpressionKind::Identifier("x".to_string()),
                                    Span::unknown(),
                                )),
                            },
                            Span::unknown(),
                        )),
                    },
                    Span::unknown(),
                );

                // Capture the current environment for the composed function
                // The call frame will be created at call time
                let composed =
                    Value::Lambda(Closure::new(params, None, Box::new(body), Rc::clone(env)));
                Ok(Rc::new(composed))
            }
            ExpressionKind::Lazy(_) => {
                // Lazy blocks are not directly evaluated - they're handled by special forms
                Err(RuntimeError::not_yet_implemented(expr.clone()))
            }
        }
    }

    pub fn apply_fn(&self, f: &ValueRef, a: &ValueRef, env: &EnvRef) -> EvalResult {
        match f.as_ref() {
            Value::Lambda(closure) => {
                let closure_env = closure.get_env();

                if closure.params.len() == 0 {
                    // Zero-param lambda: create a call frame for the body evaluation
                    let call_env = Env::new_as_ref(closure_env);
                    return self.eval_expr(&closure.body, &call_env);
                }

                let mut curried = closure.clone();
                curried.binded.push(a.clone());

                if curried.binded.len() == curried.params.len() {
                    // All params are bound - create a fresh call frame environment
                    // The call frame's parent is the closure's definition environment
                    let call_env = Env::new_as_ref(closure_env);
                    curried.bind_variables_into(&call_env)?;

                    // Evaluate the body in the call frame (not in closure.env)
                    self.eval_expand(&curried.body, &call_env)
                } else {
                    // Make a new curried lambda
                    Ok(Rc::new(Value::Lambda(curried)))
                }
            }
            Value::NativeLambda(closure) => {
                if closure.params_count == 0 {
                    return closure.exec(env.clone());
                }

                let inter = closure.inter.clone();

                // Make a new curried lambda
                let mut curried =
                    NativeClosure::new(closure.params_count, Rc::clone(&closure.logic), inter);
                curried.binded.extend_from_slice(&closure.binded);
                curried.binded.push(a.clone());

                if curried.binded.len() == closure.params_count {
                    curried.exec(env.clone())
                } else {
                    Ok(Rc::new(Value::NativeLambda(curried)))
                }
            }
            _ => Ok(f.clone()),
        }
    }

    pub fn apply_special_fn(
        &self,
        closure: &SpecialBoundClosure,
        a: &Box<Expression>,
        env: &EnvRef,
    ) -> EvalResult {
        if closure.params_count == 0 {
            return closure.exec(env.clone());
        }

        let inter = closure.inter.clone();

        // Make a new curried special lambda
        let mut curried =
            SpecialBoundClosure::new(closure.params_count, Rc::clone(&closure.logic), inter);
        curried.binded.extend_from_slice(&closure.binded);
        curried.binded.push(*a.clone());

        if curried.binded.len() == closure.params_count {
            curried.exec(env.clone())
        } else {
            Ok(Rc::new(Value::SpecialBoundForm(curried)))
        }
    }

    pub fn expand(&self, v: ValueRef, env: &EnvRef) -> EvalResult {
        match v.as_ref() {
            Value::SpecialForm(closure) => closure.exec(env.clone()),
            _ => Ok(v),
        }
    }

    pub fn eval_expand(&self, expr: &Expression, env: &EnvRef) -> EvalResult {
        self.expand(self.eval_expr(expr, env)?, env)
    }

    fn lookup(&self, name: &String, env: &EnvRef) -> EvalResult {
        env.lookup(name).map_or(
            Err(RuntimeError::undefined_variable(name.clone())),
            |val| match val.as_ref() {
                Value::Lambda(closure) => {
                    let quoted = self.ctx.borrow().quoted;
                    if closure.params.len() == 0 && !quoted {
                        let closure_env = closure.get_env();
                        let call_env = Env::new_as_ref(closure_env);
                        return self.eval_expr(&closure.body, &call_env);
                    }
                    Ok(val)
                }
                Value::NativeLambda(closure) => {
                    if closure.params_count == 0 {
                        return closure.exec(env.clone());
                    }
                    Ok(val)
                }
                _ => Ok(val),
            },
        )
    }

    fn val_compare(&self, val1: &ValueRef, val2: &ValueRef) -> Result<bool, RuntimeError> {
        Eq::run(val1, val2)?.expect_bool()
    }
}
