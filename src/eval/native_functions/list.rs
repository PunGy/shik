use crate::{
    count_args,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeContext, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
    define_native,
};
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
        Some(v) => Ok(Rc::clone(v)),
        None => native_result(Value::Null),
    }
});

native_op!(ListTail, "list.tail", [lst], {
    let lst = lst.expect_list()?;
    if lst.is_empty() {
        native_result(Value::List(vec![]))
    } else {
        native_result(Value::List(lst[1..].to_vec()))
    }
});

native_op!(ListLast, "list.last", [lst], {
    let lst = lst.expect_list()?;
    match lst.last() {
        Some(v) => Ok(Rc::clone(v)),
        None => native_result(Value::Null),
    }
});

native_op!(ListInit, "list.init", [lst], {
    let lst = lst.expect_list()?;
    if lst.is_empty() {
        native_result(Value::List(vec![]))
    } else {
        native_result(Value::List(lst[..lst.len()-1].to_vec()))
    }
});

native_op!(ListReverse, "list.reverse", [lst], {
    let lst = lst.expect_list()?;
    let reversed: Vec<ValueRef> = lst.iter().rev().cloned().collect();
    native_result(Value::List(reversed))
});

native_op!(ListConcat, "list.concat", [a, b], {
    let a = a.expect_list()?;
    let b = b.expect_list()?;
    let mut result = a.clone();
    result.extend(b.iter().cloned());
    native_result(Value::List(result))
});

native_op!(ListNth, "list.nth", [idx, lst], {
    let lst = lst.expect_list()?;
    let idx = idx.expect_number()? as usize;
    match lst.get(idx) {
        Some(v) => Ok(Rc::clone(v)),
        None => native_result(Value::Null),
    }
});

native_op!(ListIsEmpty, "list.empty?", [lst], {
    let lst = lst.expect_list()?;
    native_result(Value::Bool(lst.is_empty()))
});

native_op!(ListRange, "list.range", [start, end], {
    let start = start.expect_number()? as i64;
    let end = end.expect_number()? as i64;
    let result: Vec<ValueRef> = (start..end)
        .map(|n| Rc::new(Value::Number(n as f64)))
        .collect();
    native_result(Value::List(result))
});

native_op!(ListTake, "list.take", [n, lst], {
    let lst = lst.expect_list()?;
    let n = n.expect_number()? as usize;
    let result: Vec<ValueRef> = lst.iter().take(n).cloned().collect();
    native_result(Value::List(result))
});

native_op!(ListDrop, "list.drop", [n, lst], {
    let lst = lst.expect_list()?;
    let n = n.expect_number()? as usize;
    let result: Vec<ValueRef> = lst.iter().skip(n).cloned().collect();
    native_result(Value::List(result))
});

// Higher-order functions using NativeContext to call lambdas

native_op!(ListMap, "list.map", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut result: Vec<ValueRef> = Vec::new();
    for item in lst.iter() {
        let mapped = ctx.apply(func, item)?;
        result.push(mapped);
    }
    native_result(Value::List(result))
});

native_op!(ListFilter, "list.filter", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut result: Vec<ValueRef> = Vec::new();
    for item in lst.iter() {
        let predicate_result = ctx.apply(func, item)?;
        if predicate_result.expect_bool()? {
            result.push(Rc::clone(item));
        }
    }
    native_result(Value::List(result))
});

native_op!(ListFold, "list.fold", [init, func, lst], ctx, {
    let lst = lst.expect_list()?;
    let mut acc = Rc::clone(init);
    for item in lst.iter() {
        // Apply function to accumulator first, then to item (curried)
        let partial = ctx.apply(func, &acc)?;
        acc = ctx.apply(&partial, item)?;
    }
    Ok(acc)
});

native_op!(ListAny, "list.any", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, item)?;
        if result.expect_bool()? {
            return native_result(Value::Bool(true));
        }
    }
    native_result(Value::Bool(false))
});

native_op!(ListAll, "list.all", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, item)?;
        if !result.expect_bool()? {
            return native_result(Value::Bool(false));
        }
    }
    native_result(Value::Bool(true))
});

native_op!(ListFind, "list.find", [func, lst], ctx, {
    let lst = lst.expect_list()?;
    for item in lst.iter() {
        let result = ctx.apply(func, item)?;
        if result.expect_bool()? {
            return Ok(Rc::clone(item));
        }
    }
    native_result(Value::Null)
});

pub fn bind_list_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(ListLen, env, inter);
    define_native!(ListSum, env, inter);
    define_native!(ListHead, env, inter);
    define_native!(ListTail, env, inter);
    define_native!(ListLast, env, inter);
    define_native!(ListInit, env, inter);
    define_native!(ListReverse, env, inter);
    define_native!(ListConcat, env, inter);
    define_native!(ListNth, env, inter);
    define_native!(ListIsEmpty, env, inter);
    define_native!(ListRange, env, inter);
    define_native!(ListTake, env, inter);
    define_native!(ListDrop, env, inter);
    // Higher-order functions
    define_native!(ListMap, env, inter);
    define_native!(ListFilter, env, inter);
    define_native!(ListFold, env, inter);
    define_native!(ListAny, env, inter);
    define_native!(ListAll, env, inter);
    define_native!(ListFind, env, inter);
}
