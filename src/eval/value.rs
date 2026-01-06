use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use crate::eval::evaluator::Interpretator;
use crate::eval::utils::define_match;
use crate::{
    eval::error::RuntimeError,
    parser::{Expression, MatchPattern},
};

// ============================================================================
// Cached Common Values - Reduces allocations for frequently used values
// Thread-local storage since Rc is not Sync
// ============================================================================

thread_local! {
    /// Cached null value to avoid repeated allocations
    static NULL_VALUE: ValueRef = Rc::new(Value::Null);

    /// Cached boolean true
    static TRUE_VALUE: ValueRef = Rc::new(Value::Bool(true));

    /// Cached boolean false
    static FALSE_VALUE: ValueRef = Rc::new(Value::Bool(false));

    /// Cached zero
    static ZERO_VALUE: ValueRef = Rc::new(Value::Number(0.0));

    /// Cached one
    static ONE_VALUE: ValueRef = Rc::new(Value::Number(1.0));
}

/// Get cached null value
#[inline]
pub fn null_value() -> ValueRef {
    NULL_VALUE.with(|v| Rc::clone(v))
}

/// Get cached boolean value
#[inline]
pub fn bool_value(b: bool) -> ValueRef {
    if b {
        TRUE_VALUE.with(|v| Rc::clone(v))
    } else {
        FALSE_VALUE.with(|v| Rc::clone(v))
    }
}

/// Get cached number if it's a common value, otherwise create new
#[inline]
pub fn number_value(n: f64) -> ValueRef {
    if n == 0.0 {
        ZERO_VALUE.with(|v| Rc::clone(v))
    } else if n == 1.0 {
        ONE_VALUE.with(|v| Rc::clone(v))
    } else {
        Rc::new(Value::Number(n))
    }
}

// ============================================================================
// Weak Reference Type Aliases
// ============================================================================

/// Weak reference to environment - used in closures to break reference cycles
pub type WeakEnvRef = Weak<Env>;

/// Weak reference to interpreter - used in native closures
pub type WeakInterRef = Weak<Interpretator>;

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
    SpecialBoundForm(SpecialBoundClosure),

    Null,
}

#[derive(Debug)]
pub enum MatchContext {
    Let,
    Lambda,
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

// ============================================================================
// Native Closure - For built-in functions implemented in Rust
// Uses Weak references to break reference cycles
// ============================================================================

#[derive(Debug)]
pub struct NativeClosure {
    pub params_count: usize,
    pub binded: Vec<ValueRef>,
    pub logic: Rc<dyn NativeFn>,
    pub inter: WeakInterRef,
    pub env: WeakEnvRef,
}

// ============================================================================
// Special Closure - For special forms that receive unevaluated expressions
// ============================================================================

#[derive(Debug)]
pub struct SpecialClosure {
    pub params: Vec<Expression>,
    pub interpretator: WeakInterRef,
    pub env: WeakEnvRef,
    pub logic: Rc<dyn SpecialFn>,
}

// ============================================================================
// Special Bound Closure - For special forms with bound parameters
// ============================================================================

#[derive(Debug)]
pub struct SpecialBoundClosure {
    pub params_count: usize,
    pub binded: Vec<Expression>,
    pub logic: Rc<dyn SpecialFn>,
    pub inter: WeakInterRef,
    pub env: WeakEnvRef,
}

impl NativeClosure {
    /// Execute the native closure
    /// Returns EnvironmentDropped error if the environment was garbage collected
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = self.get_inter()?;
        let env = self.get_env()?;
        let ctx = NativeContext {
            inter: &inter,
            env: &env,
        };
        self.logic.exec(&self.binded, &ctx)
    }

    pub fn new(
        params_count: usize,
        logic: Rc<dyn NativeFn>,
        inter: Rc<Interpretator>,
        env: EnvRef,
    ) -> Self {
        Self {
            params_count,
            binded: Vec::new(),
            logic,
            inter: Rc::downgrade(&inter),
            env: Rc::downgrade(&env),
        }
    }

    /// Upgrade weak environment reference, returning error if dropped
    #[inline]
    pub fn get_env(&self) -> Result<EnvRef, RuntimeError> {
        self.env.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }

    /// Upgrade weak interpreter reference, returning error if dropped
    #[inline]
    pub fn get_inter(&self) -> Result<Rc<Interpretator>, RuntimeError> {
        self.inter.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }
}

impl SpecialClosure {
    /// Execute the special closure
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = self.get_inter()?;
        let env = self.get_env()?;
        let ctx = NativeContext {
            inter: &inter,
            env: &env,
        };
        self.logic.exec(&self.params, &ctx)
    }

    pub fn new(logic: Rc<dyn SpecialFn>, interpretator: Rc<Interpretator>, env: EnvRef) -> Self {
        Self {
            params: Vec::new(),
            logic,
            interpretator: Rc::downgrade(&interpretator),
            env: Rc::downgrade(&env),
        }
    }

    /// Upgrade weak environment reference
    #[inline]
    pub fn get_env(&self) -> Result<EnvRef, RuntimeError> {
        self.env.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }

    /// Upgrade weak interpreter reference
    #[inline]
    pub fn get_inter(&self) -> Result<Rc<Interpretator>, RuntimeError> {
        self.interpretator
            .upgrade()
            .ok_or(RuntimeError::EnvironmentDropped)
    }
}

impl SpecialBoundClosure {
    /// Execute the special bound closure
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = self.get_inter()?;
        let env = self.get_env()?;
        let ctx = NativeContext {
            inter: &inter,
            env: &env,
        };
        self.logic.exec(&self.binded, &ctx)
    }

    pub fn new(
        params_count: usize,
        logic: Rc<dyn SpecialFn>,
        inter: Rc<Interpretator>,
        env: EnvRef,
    ) -> Self {
        Self {
            params_count,
            binded: Vec::new(),
            logic,
            inter: Rc::downgrade(&inter),
            env: Rc::downgrade(&env),
        }
    }

    /// Upgrade weak environment reference
    #[inline]
    pub fn get_env(&self) -> Result<EnvRef, RuntimeError> {
        self.env.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }

    /// Upgrade weak interpreter reference
    #[inline]
    pub fn get_inter(&self) -> Result<Rc<Interpretator>, RuntimeError> {
        self.inter.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }
}

// ============================================================================
// User-defined Closure - For lambdas defined in Shik code
//
// Memory Management Strategy:
// - Closure holds a STRONG reference to its own environment (keeps it alive)
// - The Env holds a WEAK reference to its parent (breaks cycles)
//
// This breaks the cycle: Parent Env -> Closure -> Child Env -> (weak) Parent Env
// ============================================================================

#[derive(Clone, Debug)]
pub struct Closure {
    pub params: Vec<MatchPattern>,
    pub rest: Option<String>,
    pub binded: Vec<ValueRef>,
    pub body: Box<Expression>,
    /// Strong reference to the closure's own environment
    /// This environment has a weak reference to its parent
    pub env: EnvRef,
}

impl Closure {
    pub fn new(
        params: Vec<MatchPattern>,
        rest: Option<String>,
        body: Box<Expression>,
        env: EnvRef,
    ) -> Self {
        Self {
            params,
            rest,
            binded: Vec::new(),
            body,
            env,
        }
    }

    /// Get the closure's environment (always available since we hold strong ref)
    #[inline]
    pub fn get_env(&self) -> EnvRef {
        Rc::clone(&self.env)
    }

    /// Bind variables in the closure's environment
    pub fn bind_variables(&self) -> Result<(), RuntimeError> {
        for (pattern, val) in self
            .params
            .iter()
            .zip(self.binded.iter())
            .collect::<Vec<_>>()
        {
            define_match(pattern, val, &self.env, &MatchContext::Lambda)?;
        }
        Ok(())
    }
}

// ============================================================================
// Environment - Holds variable bindings
//
// Uses WEAK reference to parent to break reference cycles.
// The cycle we're breaking:
//   Global Env (strong) -> vars["my-fn"] -> Closure (strong) -> Child Env (weak) -> Global Env
// ============================================================================

#[derive(Debug)]
pub struct Env {
    /// Weak reference to parent environment - breaks reference cycles
    pub parent: Option<WeakEnvRef>,
    pub vars: RefCell<HashMap<String, ValueRef>>,
    pub help: RefCell<HashMap<String, String>>,
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
            Value::Lambda(_)
            | Value::NativeLambda(_)
            | Value::SpecialForm(_)
            | Value::SpecialBoundForm(_) => ValueType::Lambda,
            Value::Bool(_) => ValueType::Bool,
            Value::Null => ValueType::Null,
        }
    }
}

impl Env {
    /// Create a new environment with an optional parent
    /// The parent is stored as a Weak reference to break cycles
    pub fn new(parent: Option<EnvRef>) -> Self {
        Self {
            parent: parent.map(|p| Rc::downgrade(&p)),
            vars: RefCell::new(HashMap::new()),
            help: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new root environment (no parent)
    pub fn new_root() -> Self {
        Self {
            parent: None,
            vars: RefCell::new(HashMap::new()),
            help: RefCell::new(HashMap::new()),
        }
    }

    /// Get the parent environment if it still exists
    pub fn get_parent(&self) -> Option<EnvRef> {
        self.parent.as_ref().and_then(|weak| weak.upgrade())
    }

    pub fn define(&self, name: String, value: ValueRef) {
        self.vars.borrow_mut().insert(name, value);
    }

    pub fn define_help(&self, name: String, message: String) {
        self.help.borrow_mut().insert(name, message);
    }

    pub fn lookup_help(&self, key: &str) -> Option<String> {
        // First check this environment
        if let Some(msg) = self.help.borrow().get(key).cloned() {
            return Some(msg);
        }
        // Then check parent chain
        if let Some(parent) = self.get_parent() {
            return parent.lookup_help(key);
        }
        None
    }

    pub fn lookup(&self, key: &str) -> Option<ValueRef> {
        // First check this environment
        if let Some(val) = self.vars.borrow().get(key).cloned() {
            return Some(val);
        }
        // Then check parent chain
        if let Some(parent) = self.get_parent() {
            return parent.lookup(key);
        }
        None
    }

    pub fn assign(&self, name: &str, value: ValueRef) -> bool {
        // Check if variable exists in this environment
        if self.vars.borrow().contains_key(name) {
            self.vars.borrow_mut().insert(name.to_string(), value);
            return true;
        }
        // Check parent chain
        if let Some(parent) = self.get_parent() {
            return parent.assign(name, value);
        }
        false
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{}", s),
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
            Value::NativeLambda(_)
            | Value::Lambda(_)
            | Value::SpecialForm(_)
            | Value::SpecialBoundForm(_) => {
                write!(f, "Lambda function")
            }

            Value::Null => write!(f, "null"),
        }
    }
}
