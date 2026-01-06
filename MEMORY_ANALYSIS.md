# Memory Management Analysis for Shik Language Interpreter

## Language Context

**Shik is a shell scripting language** with these characteristics:
- Relatively small scripts (not long-running services)
- Burst allocations of short-lived data (read file → transform → write)
- Functional filter-map-reduce patterns
- **Immediate performance matters more than eventual optimization**

This context significantly changes our optimization strategy. We don't need a sophisticated GC - we need **fast allocation, fast deallocation, and cycle breaking**.

---

## Current Architecture Analysis

### Value System
```rust
pub type ValueRef = Rc<Value>
pub type EnvRef = Rc<Env>
```

### The Problem: Reference Cycles

Your closures create cycles that `Rc` cannot break:

```
Env ──► vars["my-fn"] ──► Closure ──► env ──► (back to Env)
```

For a shell language, this is particularly problematic because:
1. **Scripts define functions at top level** → functions live in global env
2. **Functions capture their environment** → cycle created immediately
3. **Script ends** → but cycles prevent cleanup
4. **Memory accumulates** across REPL sessions or repeated script runs

---

## Recommended Solution: Weak Environment References

For your use case, the simplest and most effective fix is using `Weak<Env>` in closures. This:
- Breaks cycles immediately
- Has minimal runtime overhead
- Requires localized code changes
- Works perfectly for short-lived scripts

### Why This Works for Shell Scripts

In shell scripts:
- **Environments are created, used, and discarded quickly**
- **Closures rarely outlive their defining scope** (unlike in long-running apps)
- **If a closure's env is dropped, it's usually a bug** (we can error clearly)

---

## Implementation Plan

### Phase 1: Break Cycles with Weak References

**Changes to [`value.rs`](src/eval/value.rs):**

```rust
use std::rc::Weak;

// User-defined closures
pub struct Closure {
    pub params: Vec<MatchPattern>,
    pub rest: Option<String>,
    pub binded: Vec<ValueRef>,
    pub body: Box<Expression>,
    pub env: Weak<Env>,  // Changed from EnvRef
}

// Native closures  
pub struct NativeClosure {
    pub params_count: usize,
    pub binded: Vec<ValueRef>,
    pub logic: Rc<dyn NativeFn>,
    pub inter: Weak<Interpretator>,  // Changed from Rc
    pub env: Weak<Env>,              // Changed from EnvRef
}

// Special closures
pub struct SpecialClosure {
    pub params: Vec<Expression>,
    pub interpretator: Weak<Interpretator>,  // Changed
    pub env: Weak<Env>,                      // Changed
    pub logic: Rc<dyn SpecialFn>,
}

pub struct SpecialBoundClosure {
    pub params_count: usize,
    pub binded: Vec<Expression>,
    pub logic: Rc<dyn SpecialFn>,
    pub inter: Weak<Interpretator>,  // Changed
    pub env: Weak<Env>,              // Changed
}
```

**Helper methods:**

```rust
impl Closure {
    pub fn get_env(&self) -> Result<EnvRef, RuntimeError> {
        self.env.upgrade().ok_or(RuntimeError::EnvironmentDropped)
    }
}
```

### Phase 2: Optimize for Burst Allocations

For filter-map-reduce patterns on large data, add **in-place operations** where possible:

**Current (allocates new list):**
```rust
native_op!(ListMap, "list.map", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut result: Vec<ValueRef> = Vec::new();  // New allocation
    for item in lst.iter() {
        let mapped = ctx.apply(func, item)?;
        result.push(mapped);
    }
    native_result(Value::List(result))
});
```

**Add in-place variant:**
```rust
native_op!(ListMapInPlace, "list.map!", [func, lst], ctx, {
    let lst_ptr = Rc::as_ptr(lst) as *mut Value;
    unsafe {
        if let Value::List(lst) = &mut *lst_ptr {
            for item in lst.iter_mut() {
                *item = ctx.apply(func, item)?;
            }
        }
    }
    Ok(Rc::clone(lst))
});
```

### Phase 3: Reduce Allocations in Hot Paths

**String interning for identifiers:**
```rust
use std::collections::HashSet;

pub struct StringInterner {
    strings: RefCell<HashSet<Rc<str>>>,
}

impl StringInterner {
    pub fn intern(&self, s: &str) -> Rc<str> {
        let mut strings = self.strings.borrow_mut();
        if let Some(existing) = strings.get(s) {
            Rc::clone(existing)
        } else {
            let rc: Rc<str> = s.into();
            strings.insert(Rc::clone(&rc));
            rc
        }
    }
}
```

**Small value optimization:**
```rust
// Cache common values
lazy_static! {
    static ref NULL: ValueRef = Rc::new(Value::Null);
    static ref TRUE: ValueRef = Rc::new(Value::Bool(true));
    static ref FALSE: ValueRef = Rc::new(Value::Bool(false));
    static ref ZERO: ValueRef = Rc::new(Value::Number(0.0));
    static ref ONE: ValueRef = Rc::new(Value::Number(1.0));
}

pub fn cached_bool(b: bool) -> ValueRef {
    if b { Rc::clone(&TRUE) } else { Rc::clone(&FALSE) }
}

pub fn cached_null() -> ValueRef {
    Rc::clone(&NULL)
}
```

---

## Specific Code Changes

### 1. Add new error variant

**[`error.rs`](src/eval/error.rs):**
```rust
pub enum RuntimeError {
    // ... existing variants ...
    
    /// Environment was dropped (closure outlived its scope)
    EnvironmentDropped,
}
```

### 2. Update Closure structures

**[`value.rs`](src/eval/value.rs):**

The key changes are:
1. Change `EnvRef` to `Weak<Env>` in all closure types
2. Change `Rc<Interpretator>` to `Weak<Interpretator>` in native closures
3. Add helper methods to upgrade weak references with proper error handling

### 3. Update evaluator

**[`evaluator.rs`](src/eval/evaluator.rs):**

When creating closures, use `Rc::downgrade()`:
```rust
Expression::Lambda { parameters, rest, body } => {
    let child_env = Rc::new(Env::new(Some(Rc::clone(env))));
    Ok(Rc::new(Value::Lambda(Closure::new(
        parameters.clone(),
        rest.clone(),
        body.clone(),
        Rc::downgrade(&child_env),
    ))))
}
```

When using closures, upgrade the weak reference:
```rust
Value::Lambda(closure) => {
    let env = closure.get_env()?;  // Returns Result, propagates error
    // ... use env ...
}
```

### 4. Update native function macros

**[`macros.rs`](src/eval/native_functions/macros.rs):**

Update the macro to use weak references when creating closures.

---

## Performance Considerations for Shell Scripts

### What Matters Most

1. **Fast list operations** - filter/map/reduce are the bread and butter
2. **Minimal allocation overhead** - burst allocations should be cheap
3. **Quick cleanup** - when script ends, memory should be freed

### What Matters Less

1. **Long-running optimization** - scripts are short-lived
2. **Sophisticated GC** - overkill for shell scripts
3. **Concurrent access** - shell scripts are typically single-threaded

### Recommended Optimizations Priority

| Priority | Optimization | Impact | Effort |
|----------|-------------|--------|--------|
| 1 | Weak references in closures | Fixes memory leaks | Medium |
| 2 | Cached common values (null, true, false) | Reduces allocations | Low |
| 3 | In-place list operations (`map!`, `filter!`) | Reduces allocations | Low |
| 4 | String interning | Reduces string allocations | Medium |
| 5 | Arena allocator for values | Bulk deallocation | High |

---

## Quick Win: Cached Values

This is the easiest optimization to implement right now:

```rust
// In value.rs or a new cache.rs

use once_cell::sync::Lazy;

pub static NULL_VALUE: Lazy<ValueRef> = Lazy::new(|| Rc::new(Value::Null));
pub static TRUE_VALUE: Lazy<ValueRef> = Lazy::new(|| Rc::new(Value::Bool(true)));
pub static FALSE_VALUE: Lazy<ValueRef> = Lazy::new(|| Rc::new(Value::Bool(false)));

// Usage in native functions:
// Instead of: native_result(Value::Null)
// Use: Ok(Rc::clone(&NULL_VALUE))
```

This alone can significantly reduce allocations in loops and conditionals.

---

## The Unsafe Code Issue

Your [`list.rs`](src/eval/native_functions/list.rs:242) has unsafe mutation:

```rust
let lst_ptr = Rc::as_ptr(lst) as *mut Value;
unsafe {
    match &mut *lst_ptr {
        Value::List(lst) => {
            lst[inx] = Rc::clone(&content);
        }
    }
}
```

This is **undefined behavior** because `Rc` assumes immutability. Two options:

**Option A: Use `Rc<RefCell<Value>>`** (safe but adds overhead)
```rust
pub type ValueRef = Rc<RefCell<Value>>;
```

**Option B: Keep unsafe but document it** (faster, your current approach)
```rust
/// SAFETY: This function mutates through Rc, which is technically UB.
/// However, in our single-threaded interpreter with no aliasing during
/// mutation, this is safe in practice. We accept this tradeoff for
/// performance in shell script use cases.
```

For a shell scripting language where performance matters and you control all access patterns, Option B is pragmatic. Just be aware of the tradeoff.

---

## Summary

For your shell scripting language:

1. **Immediate fix**: Use `Weak<Env>` in closures to break reference cycles
2. **Quick win**: Cache common values (null, true, false, small numbers)
3. **Performance boost**: Add in-place variants of list operations
4. **Future consideration**: String interning for repeated identifiers

The `Weak` reference change is the most important - it will fix your memory leak issue with minimal performance impact. The other optimizations are nice-to-haves that can be added incrementally.

Would you like me to implement these changes?
