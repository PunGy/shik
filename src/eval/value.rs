use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::rc::Rc;

use crate::eval::evaluator::Interpretator;
use crate::{
    eval::error::RuntimeError,
    parser::{Expression, MatchPattern},
};

#[derive(Debug)]
pub enum ValueType {
    Number,
    String,
    Bool,
    List,
    Object,
    Lambda,
    Null,
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<ValueRef>),
    Object(HashMap<String, ValueRef>),
    Lambda(Closure),

    NativeLambda(NativeClosure),
    SpecialForm(SpecialClosure),

    Null,
}

pub type ValueRef = Rc<Value>;

/// Context passed to native functions, providing access to the interpretator and environment
pub struct NativeContext<'a> {
    pub inter: &'a Interpretator,
    pub env: &'a EnvRef,
}

impl<'a> NativeContext<'a> {
    pub fn apply(&self, f: &ValueRef, arg: &ValueRef) -> Result<ValueRef, RuntimeError> {
        self.inter.apply_fn(f, arg)
    }
}

pub trait NativeFn: Debug {
    fn exec(&self, args: &Vec<ValueRef>, ctx: &NativeContext) -> Result<ValueRef, RuntimeError>;
}

pub trait SpecialFn: Debug {
    fn exec(&self, args: &Vec<Expression>, ctx: &NativeContext) -> Result<ValueRef, RuntimeError>;
}

#[derive(Debug)]
pub struct NativeClosure {
    pub params_count: usize,
    pub binded: Vec<ValueRef>,
    pub logic: Rc<dyn NativeFn>,
    pub inter: Rc<Interpretator>,
    pub env: EnvRef,
}
#[derive(Debug)]
pub struct SpecialClosure {
    pub params: Vec<Expression>,
    pub interpretator: Rc<Interpretator>,
    pub env: EnvRef,

    pub logic: Rc<dyn SpecialFn>,
}

impl NativeClosure {
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let ctx = NativeContext {
            inter: &self.inter,
            env: &self.env,
        };
        self.logic.exec(&self.binded, &ctx)
    }

    pub fn new(params_count: usize, logic: Rc<dyn NativeFn>, inter: Rc<Interpretator>, env: EnvRef) -> Self {
        Self {
            params_count,
            binded: Vec::new(),
            logic,
            inter,
            env,
        }
    }
}

impl SpecialClosure {
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let ctx = NativeContext {
            inter: &self.interpretator,
            env: &self.env,
        };
        self.logic.exec(&self.params, &ctx)
    }

    pub fn new(logic: Rc<dyn SpecialFn>, interpretator: Rc<Interpretator>, env: EnvRef) -> Self {
        Self {
            params: Vec::new(),
            logic,
            interpretator,
            env,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub params: Vec<MatchPattern>,
    pub binded: Vec<ValueRef>,
    pub body: Box<Expression>,
    pub env: EnvRef,
}

impl Closure {
    pub fn new(params: Vec<MatchPattern>, body: Box<Expression>, env: EnvRef) -> Self {
        Self {
            params,
            binded: Vec::new(),
            body,
            env,
        }
    }
    pub fn bind_variables(&self) {
        for (p, v) in self
            .params
            .iter()
            .zip(self.binded.iter())
            .collect::<Vec<_>>()
        {
            match &p {
                MatchPattern::Identifier(id) => {
                    self.env.define(id.clone(), Rc::clone(v));
                }
                _ => panic!("not support pattern matching yet"),
            }
        }
    }
}

#[derive(Debug)]
pub struct Env {
    pub parent: Option<EnvRef>,
    pub vars: RefCell<HashMap<String, ValueRef>>,
}

pub type EnvRef = Rc<Env>;

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(x) => Some(*x),
            _ => None,
        }
    }
    pub fn expect_number(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Number(x) => Ok(*x),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::Number,
            }),
        }
    }
    pub fn expect_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Bool(x) => Ok(*x),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::Bool,
            }),
        }
    }
    pub fn expect_string(&self) -> Result<&String, RuntimeError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::String,
            }),
        }
    }
    pub fn expect_list(&self) -> Result<&Vec<ValueRef>, RuntimeError> {
        match self {
            Value::List(lst) => Ok(lst),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::List,
            }),
        }
    }
    pub fn expect_obj(&self) -> Result<&HashMap<String, ValueRef>, RuntimeError> {
        match self {
            Value::Object(obj) => Ok(obj),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::Object,
            }),
        }
    }
    pub fn expect_native_lambda(&self) -> Result<&NativeClosure, RuntimeError> {
        match self {
            Value::NativeLambda(l) => Ok(l),
            _ => Err(RuntimeError::MissmatchedTypes {
                got: self.get_type(),
                expected: ValueType::Lambda,
            }),
        }
    }

    pub fn into_string(&self) -> Rc<Value> {
        Rc::new(Value::String(self.to_string()))
    }

    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::List(_) => ValueType::List,
            Value::Object(_) => ValueType::Object,
            Value::Lambda(_) | Value::NativeLambda(_) | Value::SpecialForm(_) => ValueType::Lambda,
            Value::Bool(_) => ValueType::Bool,
            Value::Null => ValueType::Null,
        }
    }
}

impl Env {
    pub fn new(parent: Option<EnvRef>) -> Self {
        Self {
            parent,
            vars: RefCell::new(HashMap::new()),
        }
    }

    pub fn define(&self, name: String, value: ValueRef) {
        self.vars.borrow_mut().insert(name, value);
    }

    pub fn lookup(&self, key: &str) -> Option<ValueRef> {
        iter::successors(Some(self), |env| env.parent.as_deref())
            .find_map(|env| env.vars.borrow().get(key).cloned())
    }

    pub fn assign(&self, name: &str, value: ValueRef) -> bool {
        iter::successors(Some(self), |env| env.parent.as_deref())
            .find(|env| env.vars.borrow().contains_key(name))
            .map_or(false, |e| {
                e.vars.borrow_mut().insert(name.to_string(), value);
                true
            })
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(l) => {
                write!(f, "[")?;
                for i in l.iter() {
                    write!(f, " {}", i)?;
                }
                write!(f, " ]")
            }
            Value::Object(o) => {
                write!(f, "{{")?;
                for (name, value) in o.iter() {
                    write!(f, "{}: {},\n", name, value)?;
                }
                write!(f, "}}")
            }
            Value::NativeLambda(_) | Value::Lambda(_) | Value::SpecialForm(_) => {
                write!(f, "Lambda function")
            }

            Value::Null => write!(f, "null"),
        }
    }
}
