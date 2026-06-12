use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{
            number_value, EnvRef, ListRepr, NativeClosure, NativeContext, NativeFn, SpecialClosure,
            SpecialFn, Value, ValueRef, ValueType,
        },
        EvalResult,
    },
    native_op,
    parser::Expression,
    special_op,
};
use rand::prelude::*;
use std::rc::Rc;

native_op!(ListLen, "list.len", [lst], {
    let lst = lst.expect_list()?;
    native_result(Value::Number(lst.len() as f64))
});

native_op!(ListSum, "list.sum", [lst], {
    let lst = lst.expect_list()?;
    let mut sum = 0.0;
    for item in lst.iter() {
        sum += item.expect_number()?;
    }
    native_result(Value::Number(sum))
});

native_op!(ListHead, "list.head", [lst], {
    let lst = lst.expect_list()?;
    match lst.first() {
        Some(v) => Ok(v),
        None => native_result(Value::Null),
    }
});

native_op!(ListTail, "list.tail", [lst], {
    let lst = lst.expect_list()?;
    native_result(Value::List(lst.tail()))
});

native_op!(ListLast, "list.last", [lst], {
    let lst = lst.expect_list()?;
    match lst.last() {
        Some(v) => Ok(v),
        None => native_result(Value::Null),
    }
});

native_op!(ListInit, "list.init", [lst], {
    let lst = lst.expect_list()?;
    native_result(Value::List(lst.init()))
});

// Materializing operation - creates new Vec
native_op!(ListReverse, "list.reverse", [lst], {
    let lst = lst.expect_list()?;
    let reversed: Vec<ValueRef> = lst.iter().rev().collect();
    native_result(Value::List(ListRepr::from_vec(reversed)))
});

// Materializing operation - creates new Vec
native_op!(ListConcat, "list.concat", [a, b], {
    let a = a.expect_list()?;
    let b = b.expect_list()?;
    let mut result = Vec::with_capacity(a.len() + b.len());
    result.extend(a.iter());
    result.extend(b.iter());
    native_result(Value::List(ListRepr::from_vec(result)))
});

native_op!(ListAt, "list.at", [idx, lst], {
    let lst = lst.expect_list()?;
    let idx = idx.expect_number()? as usize;
    match lst.get(idx) {
        Some(v) => Ok(v),
        None => native_result(Value::Null),
    }
});

native_op!(ListIsEmpty, "list.empty?", [lst], {
    let lst = lst.expect_list()?;
    native_result(Value::Bool(lst.is_empty()))
});

special_op!(ListRange, "list.range", args, ctx, {
    let mut start = 0;
    let end;
    let mut step = 1;

    match args.len() {
        1 => {
            end = ctx.inter.eval_expand(&args[0], ctx.env)?.expect_number()? as i64;
        }
        2 => {
            start = ctx.inter.eval_expand(&args[0], ctx.env)?.expect_number()? as i64;
            end = ctx.inter.eval_expand(&args[1], ctx.env)?.expect_number()? as i64;
        }
        3 => {
            start = ctx.inter.eval_expand(&args[0], ctx.env)?.expect_number()? as i64;
            end = ctx.inter.eval_expand(&args[1], ctx.env)?.expect_number()? as i64;
            step = ctx.inter.eval_expand(&args[2], ctx.env)?.expect_number()? as usize;
        }
        count => {
            return Err(RuntimeError::invalid_application(format!(
                "(list.range) wrong number of arguments. Must be 1, 2 or 3. Got {}",
                count
            )))
        }
    }

    // Pre-allocate with known size
    let len = if end > start && step > 0 {
        ((end - start) as usize).div_ceil(step)
    } else {
        0
    };
    let mut result = Vec::with_capacity(len);

    // Use cached number values for 0 and 1, create new for others
    for n in (start..end).step_by(step) {
        result.push(number_value(n as f64));
    }
    native_result(Value::List(ListRepr::from_vec(result)))
});

native_op!(ListTake, "list.take", [n, lst], {
    let lst = lst.expect_list()?;
    let n = n.expect_number()? as usize;
    native_result(Value::List(lst.take(n)))
});

native_op!(ListDrop, "list.drop", [n, lst], {
    let lst = lst.expect_list()?;
    let n = n.expect_number()? as usize;
    native_result(Value::List(lst.drop(n)))
});

// Higher-order functions using NativeContext to call lambdas

// Materializing operation - creates new Vec
native_op!(ListMap, "list.map", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut result: Vec<ValueRef> = Vec::with_capacity(lst.len());
    for item in lst.iter() {
        let mapped = ctx.apply(func, &item, ctx.env)?;
        result.push(mapped);
    }
    native_result(Value::List(ListRepr::from_vec(result)))
});

native_op!(ListIterate, "list.iterate", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        ctx.apply(func, &item, ctx.env)?;
    }
    native_result(Value::Null)
});

native_op!(
    ListIterateBackward,
    ["list.iterate-backward", "list.<iterate"],
    [func, lst],
    ctx,
    {
        let lst = lst.expect_list()?;
        for item in lst.iter().rev() {
            ctx.apply(func, &item, ctx.env)?;
        }
        native_result(Value::Null)
    }
);

// Materializing operation - creates new Vec
native_op!(ListFilter, "list.filter", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut result: Vec<ValueRef> = Vec::with_capacity(lst.len());
    for item in lst.iter() {
        let predicate_result = ctx.apply(func, &item, ctx.env)?;
        if predicate_result.expect_bool()? {
            result.push(item);
        }
    }
    native_result(Value::List(ListRepr::from_vec(result)))
});

native_op!(ListFold, "list.fold", [init, func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut acc = Rc::clone(init);
    for item in lst.iter() {
        // Apply function to accumulator first, then to item (curried)
        let partial = ctx.apply(func, &acc, ctx.env)?;
        acc = ctx.apply(&partial, &item, ctx.env)?;
    }
    Ok(acc)
});

native_op!(ListAny, "list.any", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, &item, ctx.env)?;
        if result.expect_bool()? {
            return native_result(Value::Bool(true));
        }
    }
    native_result(Value::Bool(false))
});

native_op!(ListAll, "list.all", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, &item, ctx.env)?;
        if !result.expect_bool()? {
            return native_result(Value::Bool(false));
        }
    }
    native_result(Value::Bool(true))
});

native_op!(ListFind, "list.find", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, &item, ctx.env)?;
        if result.expect_bool()? {
            return Ok(item);
        }
    }
    native_result(Value::Null)
});

native_op!(ListFindIndex, "list.find-index", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for (inx, item) in lst.iter().enumerate() {
        let result = ctx.apply(func, &item, ctx.env)?;
        if result.expect_bool()? {
            return native_result(Value::Number(inx as f64));
        }
    }
    native_result(Value::Number(-1.0))
});

// ============================================================================
// Mutable List Operations
//
// These operations use RefCell for safe interior mutability.
// Views are automatically materialized before mutation (COW semantics).
// ============================================================================

native_op!(ListSet, "list.set", [inx, lst, content], {
    let inx = inx.expect_number()? as usize;

    // Get mutable access to the list through the Value
    let lst_ptr = Rc::as_ptr(lst) as *mut Value;
    // SAFETY: Single-threaded interpreter, we're the only accessor
    unsafe {
        match &mut *lst_ptr {
            Value::List(list_repr) => {
                list_repr.set(inx, Rc::clone(content))?;
                return Ok(Rc::clone(content));
            }
            _ => {
                return Err(RuntimeError::mismatched_types(
                    lst.get_type(),
                    ValueType::List,
                ))
            }
        }
    }
});

native_op!(
    ListPush,
    ["list.push", "list.push>", "list.push-right"],
    [lst, content],
    {
        // Get mutable access to the list through the Value
        let lst_ptr = Rc::as_ptr(lst) as *mut Value;
        // SAFETY: Single-threaded interpreter, we're the only accessor
        unsafe {
            match &mut *lst_ptr {
                Value::List(list_repr) => {
                    list_repr.push(Rc::clone(content));
                    return Ok(Rc::clone(content));
                }
                _ => {
                    return Err(RuntimeError::mismatched_types(
                        lst.get_type(),
                        ValueType::List,
                    ))
                }
            }
        }
    }
);

native_op!(
    ListPushLeft,
    ["list.<push", "list.push-left"],
    [lst, content],
    {
        // Get mutable access to the list through the Value
        let lst_ptr = Rc::as_ptr(lst) as *mut Value;
        // SAFETY: Single-threaded interpreter, we're the only accessor
        unsafe {
            match &mut *lst_ptr {
                Value::List(list_repr) => {
                    list_repr.push_front(Rc::clone(content));
                    return Ok(Rc::clone(content));
                }
                _ => {
                    return Err(RuntimeError::mismatched_types(
                        lst.get_type(),
                        ValueType::List,
                    ))
                }
            }
        }
    }
);

native_op!(ListSlice, "list.slice", [start, end, lst], {
    let lst = lst.expect_list()?;
    let start = start.expect_number()? as usize;
    let end = end.expect_number()? as usize;
    let result = lst.slice(start, end);

    native_result(Value::List(result))
});

// Sort a list using a comparator function
// The comparator takes two arguments and returns a number:
// - negative if a < b
// - zero if a == b
// - positive if a > b
// Usage: list.sort (fn [a b] (- a b)) [3 1 2]  ; [1 2 3]
native_op!(ListSort, "list.sort", [func, lst], ctx, {
    let lst = lst.expect_list()?;

    // Materialize the list into a Vec for sorting
    let mut items: Vec<ValueRef> = lst.iter().collect();

    // We need to sort with a fallible comparator, so we use a cell to capture any error
    let mut sort_error: Option<RuntimeError> = None;

    items.sort_by(|a, b| {
        // If we already have an error, don't do more comparisons
        if sort_error.is_some() {
            return std::cmp::Ordering::Equal;
        }

        // Apply the comparator function: func a b
        let partial = match ctx.apply(func, a, ctx.env) {
            Ok(p) => p,
            Err(e) => {
                sort_error = Some(e);
                return std::cmp::Ordering::Equal;
            }
        };

        let result = match ctx.apply(&partial, b, ctx.env) {
            Ok(r) => r,
            Err(e) => {
                sort_error = Some(e);
                return std::cmp::Ordering::Equal;
            }
        };

        // Extract the number result
        let cmp_value = match result.expect_number() {
            Ok(n) => n,
            Err(e) => {
                sort_error = Some(e);
                return std::cmp::Ordering::Equal;
            }
        };

        // Convert to Ordering
        if cmp_value < 0.0 {
            std::cmp::Ordering::Less
        } else if cmp_value > 0.0 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    // Check if there was an error during sorting
    if let Some(e) = sort_error {
        return Err(e);
    }

    native_result(Value::List(ListRepr::from_vec(items)))
});

// Get a random element from a list
native_op!(ListRandGet, "list.choice", [lst], {
    let lst = lst.expect_list()?;

    if lst.is_empty() {
        return native_result(Value::Null);
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..lst.len());

    match lst.get(idx) {
        Some(v) => Ok(v),
        None => native_result(Value::Null),
    }
});

pub fn bind_list_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "list.".to_string(),
        "list module:

- list.set: sets element at index (mutates list)
- list.push, list.push>, list.push-right: appends value to end (mutates list)
- list.<push, list.push-left: prepends value to start (mutates list)
- list.at: gets element at index
- list.len: returns length
- list.sum: sums all numbers
- list.head: first element
- list.tail: all but first element
- list.last: last element
- list.init: all but last element
- list.reverse: reverses list
- list.concat: concatenates two lists
- list.empty?: checks if empty
- list.range: creates range list
- list.take: takes first n elements
- list.drop: drops first n elements
- list.slice: extracts sublist
- list.map: transforms each element
- list.filter: keeps elements matching predicate
- list.fold: reduces to single value
- list.any: true if any matches
- list.all: true if all match
- list.find: finds first matching element
- list.find-index: finds index of first match
- list.iterate: iterates forward
- list.<iterate, list.iterate-backward: iterates backward
- list.sort: sorts list using comparator function
- list.choice: gets random element from list"
            .to_string(),
    );

    define_native!(ListSet, env, inter);
    define_help!(
        ListSet,
        env,
        "[index:number list value]: sets element at index (mutates list)\n\nlist.set 0 mylist \"new\""
    );

    define_native!(ListPush, env, inter);
    define_help!(
        ListPush,
        env,
        "[list value]: appends value to end of list (mutates list)\n\nlist.push mylist 42"
    );

    define_native!(ListPushLeft, env, inter);
    define_help!(
        ListPushLeft,
        env,
        "[list value]: prepends value to start of list (mutates list)\n\nlist.<push mylist 42"
    );

    define_native!(ListAt, env, inter);
    define_help!(ListAt, env, "[index:number list]: returns element at index, or null if out of bounds\n\nlist.at 0 [1 2 3]  ; 1");

    define_native!(ListLen, env, inter);
    define_help!(
        ListLen,
        env,
        "[list]: returns length of list\n\nlist.len [1 2 3]  ; 3"
    );

    define_native!(ListSum, env, inter);
    define_help!(
        ListSum,
        env,
        "[list]: returns sum of all numbers in list\n\nlist.sum [1 2 3]  ; 6"
    );

    define_native!(ListHead, env, inter);
    define_help!(
        ListHead,
        env,
        "[list]: returns first element, or null if empty\n\nlist.head [1 2 3]  ; 1"
    );

    define_native!(ListTail, env, inter);
    define_help!(
        ListTail,
        env,
        "[list]: returns list without first element \n\nlist.tail [1 2 3]  ; [2 3]"
    );

    define_native!(ListLast, env, inter);
    define_help!(
        ListLast,
        env,
        "[list]: returns last element, or null if empty\n\nlist.last [1 2 3]  ; 3"
    );

    define_native!(ListInit, env, inter);
    define_help!(
        ListInit,
        env,
        "[list]: returns list without last element \n\nlist.init [1 2 3]  ; [1 2]"
    );

    define_native!(ListReverse, env, inter);
    define_help!(
        ListReverse,
        env,
        "[list]: returns reversed list\n\nlist.reverse [1 2 3]  ; [3 2 1]"
    );

    define_native!(ListConcat, env, inter);
    define_help!(
        ListConcat,
        env,
        "[list list]: concatenates two lists\n\nlist.concat [1 2] [3 4]  ; [1 2 3 4]"
    );

    define_native!(ListIsEmpty, env, inter);
    define_help!(
        ListIsEmpty,
        env,
        "[list]: returns true if list is empty\n\nlist.empty? []  ; true"
    );

    define_native!(ListRange, env, inter);
    define_help!(ListRange, env, "[end:number] or [start:number end:number] or [start:number end:number step:number]: creates range list\n\nlist.range 5  ; [0 1 2 3 4]\nlist.range 2 5  ; [2 3 4]\nlist.range 0 10 2  ; [0 2 4 6 8]");

    define_native!(ListTake, env, inter);
    define_help!(
        ListTake,
        env,
        "[count:number list]: takes first n elements \n\nlist.take 2 [1 2 3 4]  ; [1 2]"
    );

    define_native!(ListDrop, env, inter);
    define_help!(
        ListDrop,
        env,
        "[count:number list]: drops first n elements \n\nlist.drop 2 [1 2 3 4]  ; [3 4]"
    );

    // Higher-order functions
    define_native!(ListMap, env, inter);
    define_help!(ListMap, env, "[mapper:lambda list]: applies function to each element, returns new list\n\nlist.map (fn [x] (* x 2)) [1 2 3]  ; [2 4 6]");

    define_native!(ListFilter, env, inter);
    define_help!(ListFilter, env, "[predicate:lambda list]: keeps elements where predicate returns true\n\nlist.filter (fn [x] (> x 2)) [1 2 3 4]  ; [3 4]");

    define_native!(ListFold, env, inter);
    define_help!(ListFold, env, "[initial:value reducer:lambda list]: reduces list to single value using accumulator\n\nlist.fold 0 + [1 2 3]  ; 6");

    define_native!(ListAny, env, inter);
    define_help!(ListAny, env, "[predicate:lambda list]: returns true if any element satisfies predicate\n\nlist.any (fn [x] (> x 5)) [1 2 6]  ; true");

    define_native!(ListAll, env, inter);
    define_help!(ListAll, env, "[predicate:lambda list]: returns true if all elements satisfy predicate\n\nlist.all (fn [x] (> x 0)) [1 2 3]  ; true");

    define_native!(ListFind, env, inter);
    define_help!(ListFind, env, "[predicate:lambda list]: returns first element satisfying predicate, or null\n\nlist.find (fn [x] (> x 2)) [1 2 3 4]  ; 3");

    define_native!(ListFindIndex, env, inter);
    define_help!(ListFindIndex, env, "[predicate:lambda list]: returns index of first element satisfying predicate, or -1\n\nlist.find-index (fn [x] (> x 2)) [1 2 3 4]  ; 2");

    define_native!(ListIterate, env, inter);
    define_help!(ListIterate, env, "[callback:lambda list]: calls function for each element (for side effects)\n\nlist.iterate print [1 2 3]");

    define_native!(ListIterateBackward, env, inter);
    define_help!(ListIterateBackward, env, "[callback:lambda list]: iterates list in reverse order\n\nlist.<iterate print [1 2 3]  ; prints 3, 2, 1");

    define_native!(ListSlice, env, inter);
    define_help!(ListSlice, env, "[start:number end:number list]: extracts sublist from start(inclusice) to end(non-inclusive) index\n\nlet lst [1 2 3 4]\nlist.slice 0 2 lst ; [1 2]");

    define_native!(ListSort, env, inter);
    define_help!(ListSort, env, "[comparator:lambda list]: sorts list using comparator function. Comparator takes two arguments and returns a number (negative if a < b, zero if equal, positive if a > b)\n\nlist.sort (fn [a b] (- a b)) [3 1 2]  ; [1 2 3]\nlist.sort (fn [a b] (- b a)) [3 1 2]  ; [3 2 1]");

    define_native!(ListRandGet, env, inter);
    define_help!(ListRandGet, env, "[list]: returns a random element from the list, or null if empty\n\nlist.choice [1 2 3 4 5]  ; random element");
}
