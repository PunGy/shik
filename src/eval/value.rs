use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::rc::Rc;
use std::cell::RefCell;

use crate::{
    eval::error::RuntimeError,
    parser::{Expression, MatchPattern},
};

#[derive(Debug)]
pub enum ValueType {
    Number,
    String,
    List,
    Object,
    Lambda,
    Null,
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    List(Vec<ValueRef>),
    Object(HashMap<String, ValueRef>),
    Lambda(Rc<Closure>),

    NativeLambda(Rc<NativeClosure>),

    Null,
}

pub type ValueRef = Rc<Value>;

pub trait NativeFn: Debug {
    fn exec(&self, args: &Vec<ValueRef>) -> Result<ValueRef, RuntimeError>;
}

#[derive(Debug)]
pub struct NativeClosure {
    pub params_count: usize,
    pub binded: Vec<ValueRef>,

    pub logic: Rc<dyn NativeFn>,
}

impl NativeClosure {
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        self.logic.exec(&self.binded)
    }

    pub fn new(params_count: usize, logic: Rc<dyn NativeFn>) -> Self {
        Self {
            params_count,
            binded: Vec::new(),
            logic,
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
    pub fn bind_variables(&self, ctx: EnvRef) {
        for (p, v) in self
            .params
            .iter()
            .zip(self.binded.iter())
            .collect::<Vec<_>>()
        {
            match &p {
                MatchPattern::Identifier(id) => {
                    ctx.assign(id, Rc::clone(v));
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

    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::List(_) => ValueType::List,
            Value::Object(_) => ValueType::Object,
            Value::Lambda(_) => ValueType::Lambda,
            Value::NativeLambda(_) => ValueType::Lambda,
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
