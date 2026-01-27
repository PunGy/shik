use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::{Rc};

use crate::eval::evaluator::Interpretator;
use crate::eval::utils::define_match;
use crate::{
    eval::error::RuntimeError,
    parser::{Expression, MatchPattern},
};

// ============================================================================
// ListRepr - Efficient list representation with views and safe mutability
//
// Design goals:
// 1. O(1) tail/init/take/drop operations via views (no copying)
// 2. Safe mutability using RefCell
// 3. Copy-on-write semantics for mutating views
// ============================================================================

/// Shared list data storage
pub type ListData = Rc<RefCell<Vec<ValueRef>>>;

/// List representation with view support
///
/// A view is a slice into a shared buffer. Multiple ListRepr instances
/// can share the same underlying data with different start/end indices.
#[derive(Clone, Debug)]
pub struct ListRepr {
    /// Shared reference to the underlying data
    pub data: ListData,
    /// Start index of the view (inclusive)
    pub start: usize,
    /// End index of the view (exclusive)
    pub end: usize,
}

impl ListRepr {
    /// Create a new ListRepr from a vector
    pub fn from_vec(v: Vec<ValueRef>) -> Self {
        let len = v.len();
        Self {
            data: Rc::new(RefCell::new(v)),
            start: 0,
            end: len,
        }
    }

    /// Create an empty list
    pub fn empty() -> Self {
        Self {
            data: Rc::new(RefCell::new(Vec::new())),
            start: 0,
            end: 0,
        }
    }

    /// Get the length of this view
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this view is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this is a full view (not a slice)
    #[inline]
    pub fn is_full_view(&self) -> bool {
        self.start == 0 && self.end == self.data.borrow().len()
    }

    /// Get the first element
    pub fn first(&self) -> Option<ValueRef> {
        self.get(0)
    }

    /// Get the last element
    pub fn last(&self) -> Option<ValueRef> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len() - 1)
        }
    }

    /// Get an element by index (relative to view start)
    pub fn get(&self, idx: usize) -> Option<ValueRef> {
        let i = self.start + idx;
        if i < self.end {
            Some(Rc::clone(&self.data.borrow()[i]))
        } else {
            None
        }
    }

    /// Execute a function with a borrowed slice of the view
    pub fn with_slice<R>(&self, f: impl FnOnce(&[ValueRef]) -> R) -> R {
        let borrow = self.data.borrow();
        f(&borrow[self.start..self.end])
    }

    /// Create a new view as a slice of this view
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let new_start = (self.start + start).min(self.end);
        let new_end = (self.start + end).min(self.end).max(new_start);
        Self {
            data: Rc::clone(&self.data),
            start: new_start,
            end: new_end,
        }
    }

    /// Get tail (all elements except first) - O(1)
    pub fn tail(&self) -> Self {
        if self.is_empty() {
            self.clone()
        } else {
            self.slice(1, self.len())
        }
    }

    /// Get init (all elements except last) - O(1)
    pub fn init(&self) -> Self {
        if self.is_empty() {
            self.clone()
        } else {
            self.slice(0, self.len() - 1)
        }
    }

    /// Take first n elements - O(1)
    pub fn take(&self, n: usize) -> Self {
        self.slice(0, n.min(self.len()))
    }

    /// Drop first n elements - O(1)
    pub fn drop(&self, n: usize) -> Self {
        self.slice(n.min(self.len()), self.len())
    }

    /// Materialize the view into a new owned vector
    /// This is used before mutations on non-full views
    pub fn materialize(&self) -> Vec<ValueRef> {
        self.with_slice(|s| s.iter().cloned().collect())
    }

    /// Ensure this is a full view, materializing if necessary
    /// Returns a ListRepr that is safe to mutate
    pub fn ensure_owned(&mut self) {
        if !self.is_full_view() {
            let materialized = self.materialize();
            self.data = Rc::new(RefCell::new(materialized));
            self.start = 0;
            self.end = self.data.borrow().len();
        }
    }

    /// Set an element at index (relative to view start)
    /// Materializes the view if it's not a full view
    pub fn set(&mut self, idx: usize, value: ValueRef) -> Result<(), RuntimeError> {
        let i = self.start + idx;
        if i >= self.end {
            return Err(RuntimeError::IndexOutOfBounds { index: idx });
        }

        // If this is not a full view, materialize first
        if !self.is_full_view() {
            self.ensure_owned();
        }

        self.data.borrow_mut()[self.start + idx] = value;
        Ok(())
    }

    /// Push an element to the end
    /// Materializes the view if it's not a full view
    pub fn push(&mut self, value: ValueRef) {
        if !self.is_full_view() {
            self.ensure_owned();
        }
        self.data.borrow_mut().push(value);
        self.end += 1;
    }

    /// Push an element to the front
    /// Materializes the view if it's not a full view
    pub fn push_front(&mut self, value: ValueRef) {
        if !self.is_full_view() {
            self.ensure_owned();
        }
        self.data.borrow_mut().insert(0, value);
        self.end += 1;
    }

    /// Create an iterator over the view
    pub fn iter(&self) -> ListIter<'_> {
        ListIter {
            data: self.data.borrow(),
            start: self.start,
            end: self.end,
            current_front: self.start,
            current_back: self.end,
        }
    }
}

/// Iterator over a ListRepr view
pub struct ListIter<'a> {
    data: Ref<'a, Vec<ValueRef>>,
    start: usize,
    end: usize,
    current_front: usize,
    current_back: usize,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = ValueRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_front < self.current_back {
            let item = Rc::clone(&self.data[self.current_front]);
            self.current_front += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.current_back - self.current_front;
        (remaining, Some(remaining))
    }
}

impl<'a> DoubleEndedIterator for ListIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_back > self.current_front {
            self.current_back -= 1;
            Some(Rc::clone(&self.data[self.current_back]))
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for ListIter<'a> {}

impl PartialEq for ListRepr {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        // Compare element by element using Rc::ptr_eq for efficiency
        // or fall back to value comparison
        self.with_slice(|a| {
            other.with_slice(|b| a.iter().zip(b.iter()).all(|(x, y)| Rc::ptr_eq(x, y)))
        })
    }
}

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
    List(ListRepr),
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
        self.inter.expand(self.inter.apply_fn(f, arg)?)
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
// ============================================================================

#[derive(Debug)]
pub struct NativeClosure {
    pub params_count: usize,
    pub binded: Vec<ValueRef>,
    pub logic: Rc<dyn NativeFn>,
    pub inter: Rc<Interpretator>,
    pub env: EnvRef,
}

// ============================================================================
// Special Closure - For special forms that receive unevaluated expressions
// ============================================================================

#[derive(Debug)]
pub struct SpecialClosure {
    pub params: Vec<Expression>,
    pub interpretator: Rc<Interpretator>,
    pub env: EnvRef,
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
    pub inter: Rc<Interpretator>,
    pub env: EnvRef,
}

impl NativeClosure {
    /// Execute the native closure
    /// Returns EnvironmentDropped error if the environment was garbage collected
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = &self.inter;
        let env = &self.env;
        let ctx = NativeContext {
            inter: inter,
            env: env,
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
            inter,
            env,
        }
    }
}

impl SpecialClosure {
    /// Execute the special closure
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = &self.interpretator;
        let env = &self.env;
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
            interpretator,
            env,
        }
    }
}

impl SpecialBoundClosure {
    /// Execute the special bound closure
    pub fn exec(&self) -> Result<Rc<Value>, RuntimeError> {
        let inter = &self.inter;
        let env = &self.env;
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
            inter,
            env,
        }
    }
}

// ============================================================================
// User-defined Closure for lambdas
// ============================================================================

#[derive(Clone, Debug)]
pub struct Closure {
    pub params: Vec<MatchPattern>,
    pub rest: Option<String>,
    pub binded: Vec<ValueRef>,
    pub body: Box<Expression>,
    pub env: Rc<Env>,
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

    /// Get the closure's environment
    #[inline]
    pub fn get_env(&self) -> EnvRef {
        Rc::clone(&self.env)
    }

    /// Bind variables into an explicit target environment
    pub fn bind_variables_into(&self, target_env: &EnvRef) -> Result<(), RuntimeError> {
        for (pattern, val) in self.params.iter().zip(self.binded.iter()) {
            define_match(pattern, val, target_env, &MatchContext::Lambda)?;
        }
        Ok(())
    }
}

// ============================================================================
// Environment - Holds variable bindings
// ============================================================================

#[derive(Debug)]
pub struct Env {
    pub parent: Option<EnvRef>,
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
    pub fn expect_list(&self) -> Result<&ListRepr, RuntimeError> {
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
    pub fn new(parent: Option<EnvRef>) -> Self {
        Self {
            parent: parent.map(|p| Rc::clone(&p)),
            vars: RefCell::new(HashMap::new()),
            help: RefCell::new(HashMap::new()),
        }
    }

    /// Create new environment and return a ref
    pub fn new_as_ref(parent: EnvRef) -> Rc<Self> {
        Rc::new(Env::new(Some(parent)))
    }

    /// Create a new root environment (no parent)
    pub fn new_root() -> Self {
        Self {
            parent: None,
            vars: RefCell::new(HashMap::new()),
            help: RefCell::new(HashMap::new()),
        }
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
        if let Some(parent) = &self.parent {
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
        if let Some(parent) = &self.parent {
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
        if let Some(parent) = &self.parent {
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
