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

native_op!(MakeString, "string", [x], {
    native_result(match x.as_ref() {
        Value::Number(n) => Value::String(n.to_string()),
        Value::String(s) => Value::String(s.clone()),
        Value::Object(_) => Value::String("[object]".to_string()),
        Value::List(_) => Value::String("[list]".to_string()),
        Value::Null => Value::String("[null]".to_string()),
        Value::Lambda(_) | Value::SpecialForm(_) | Value::NativeLambda(_) => {
            Value::String("[lambda]".to_string())
        }
        _ => Value::String("".to_string()),
    })
});

native_op!(StringSplit, "string.split", [with, str], {
    let str = str.expect_string()?;
    let with = with.expect_string()?;

    let res = str
        .split(with)
        .map(|s| Rc::new(Value::String(s.to_string())))
        .collect::<Vec<ValueRef>>();

    native_result(Value::List(res))
});

native_op!(StringConcat, "string.concat", [a, b], {
    let a = a.expect_string()?;
    let b = b.expect_string()?;
    native_result(Value::String(format!("{}{}", a, b)))
});

native_op!(StringTrim, "string.trim", [s], {
    let s = s.expect_string()?;
    native_result(Value::String(s.trim().to_string()))
});

native_op!(StringTrimStart, "string.trim-start", [s], {
    let s = s.expect_string()?;
    native_result(Value::String(s.trim_start().to_string()))
});

native_op!(StringTrimEnd, "string.trim-end", [s], {
    let s = s.expect_string()?;
    native_result(Value::String(s.trim_end().to_string()))
});

native_op!(StringUppercase, "string.upper", [s], {
    let s = s.expect_string()?;
    native_result(Value::String(s.to_uppercase()))
});

native_op!(StringLowercase, "string.lower", [s], {
    let s = s.expect_string()?;
    native_result(Value::String(s.to_lowercase()))
});

native_op!(StringContains, "string.has", [needle, haystack], {
    let haystack = haystack.expect_string()?;
    let needle = needle.expect_string()?;
    native_result(Value::Bool(haystack.contains(needle.as_str())))
});

native_op!(StringStartsWith, "string.starts-with", [prefix, s], {
    let s = s.expect_string()?;
    let prefix = prefix.expect_string()?;
    native_result(Value::Bool(s.starts_with(prefix.as_str())))
});

native_op!(StringEndsWith, "string.ends-with", [suffix, s], {
    let s = s.expect_string()?;
    let suffix = suffix.expect_string()?;
    native_result(Value::Bool(s.ends_with(suffix.as_str())))
});

native_op!(StringReplace, "string.replace", [from, to, s], {
    let s = s.expect_string()?;
    let from = from.expect_string()?;
    let to = to.expect_string()?;
    native_result(Value::String(s.replace(from.as_str(), to.as_str())))
});

native_op!(StringLength, "string.len", [s], {
    let s = s.expect_string()?;
    native_result(Value::Number(s.len() as f64))
});

native_op!(StringCharAt, "string.at", [idx, s], {
    let s = s.expect_string()?;
    let idx = idx.expect_number()? as usize;
    let ch = s.chars().nth(idx);
    match ch {
        Some(c) => native_result(Value::String(c.to_string())),
        None => native_result(Value::Null),
    }
});

native_op!(StringSubstring, "string.slice", [start, end, s], {
    let s = s.expect_string()?;
    let start = start.expect_number()? as usize;
    let end = end.expect_number()? as usize;
    let result: String = s.chars().skip(start).take(end - start).collect();
    native_result(Value::String(result))
});

native_op!(StringIndexOf, "string.index-of", [needle, haystack], {
    let haystack = haystack.expect_string()?;
    let needle = needle.expect_string()?;
    match haystack.find(needle.as_str()) {
        Some(idx) => native_result(Value::Number(idx as f64)),
        None => native_result(Value::Number(-1.0)),
    }
});

native_op!(StringJoin, "string.join", [sep, lst], {
    let lst = lst.expect_list()?;
    let sep = sep.expect_string()?;
    let strings: Result<Vec<String>, _> = lst
        .iter()
        .map(|v| v.expect_string().map(|s| s.clone()))
        .collect();
    let strings = strings?;
    native_result(Value::String(strings.join(sep.as_str())))
});

native_op!(StringLines, "string.lines", [s], {
    let s = s.expect_string()?;
    let lines: Vec<ValueRef> = s
        .lines()
        .map(|line| Rc::new(Value::String(line.to_string())))
        .collect();
    native_result(Value::List(lines))
});

pub fn bind_string_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(MakeString, env, inter);
    define_native!(StringSplit, env, inter);
    define_native!(StringConcat, env, inter);
    define_native!(StringTrim, env, inter);
    define_native!(StringTrimStart, env, inter);
    define_native!(StringTrimEnd, env, inter);
    define_native!(StringUppercase, env, inter);
    define_native!(StringLowercase, env, inter);
    define_native!(StringContains, env, inter);
    define_native!(StringStartsWith, env, inter);
    define_native!(StringEndsWith, env, inter);
    define_native!(StringReplace, env, inter);
    define_native!(StringLength, env, inter);
    define_native!(StringCharAt, env, inter);
    define_native!(StringSubstring, env, inter);
    define_native!(StringIndexOf, env, inter);
    define_native!(StringJoin, env, inter);
    define_native!(StringLines, env, inter);
}
