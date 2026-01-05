use crate::{
    count_args, define_native, define_help,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeContext, NativeFn, Value, ValueRef, ValueType},
        EvalResult,
    },
    native_op,
};
use std::rc::Rc;

native_op!(MakeString, "string", [x], {
    native_result(match x.as_ref() {
        Value::Number(n) => Value::String(n.to_string()),
        Value::String(s) => Value::String(s.clone()),
        _ => Value::String(x.to_string()),
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

native_op!(StringConcat, "string.+", [a, b], {
    let a = a.expect_string()?;
    let b = b.expect_string()?;
    native_result(Value::String(format!("{}{}", a, b)))
});

native_op!(StringEq, "string.=", [a, b], {
    let a = a.expect_string()?;
    let b = b.expect_string()?;

    native_result(Value::Bool(a == b))
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
    native_result(Value::Number(s.chars().count() as f64))
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

const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
native_op!(StringBytes, "string.bytes", [b], {
    let bytes = b.expect_number()?;

    if bytes < 1024.0 {
        return native_result(Value::String("{bytes} B".to_string()));
    }

    let mut value = bytes;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    // Use one decimal place for small values, no decimals for larger ones.
    if value < 10.0 {
        return native_result(Value::String(format!("{value:.1} {}", UNITS[unit])));
    } else {
        return native_result(Value::String(format!("{value:.0} {}", UNITS[unit])));
    }
});

native_op!(StringIterate, "string.iterate", [func, str], ctx, {
    let str = str.expect_string()?;
    for char in str.chars() {
        let char = Rc::new(Value::String(char.to_string()));
        ctx.apply(func, &char)?;
    }
    native_result(Value::Null)
});

native_op!(StringIterateBackward, ["string.iterate-backward", "string.<iterate"], [func, str], ctx, {
    let str = str.expect_string()?;
    for char in str.chars().rev() {
        let char = Rc::new(Value::String(char.to_string()));
        ctx.apply(func, &char)?;
    }
    native_result(Value::Null)
});

// Helper: convert a *character index* into a UTF-8 byte range (start..end)
fn char_byte_range(s: &str, char_index: usize) -> Option<(usize, usize)> {
    let mut it = s.char_indices();
    let (start, _) = it.nth(char_index)?;
    let end = it.next().map(|(i, _)| i).unwrap_or_else(|| s.len());
    Some((start, end))
}

native_op!(StringSet, "string.set", [inx, s, content], {
    let inx = inx.expect_number()? as usize;
    let replacement = content.expect_string()?; // must be a string

    let s_ptr = Rc::as_ptr(s) as *mut Value;

    unsafe {
        match &mut *s_ptr {
            Value::String(st) => {
                // Replace the single character at `inx` with `replacement`
                let (start, end) = char_byte_range(st, inx)
                    .ok_or(RuntimeError::IndexOutOfBounds { index: inx })?;

                st.replace_range(start..end, replacement);
                Ok(Rc::clone(&content))
            }
            _ => Err(RuntimeError::MissmatchedTypes {
                got: s.get_type(),
                expected: ValueType::String,
            }),
        }
    }
});

native_op!(
    StringPush,
    ["string.push", "string.push>", "string.push-right"],
    [s, content],
    {
        let suffix = content.expect_string()?; // must be a string

        let s_ptr = Rc::as_ptr(s) as *mut Value;

        unsafe {
            match &mut *s_ptr {
                Value::String(st) => {
                    st.push_str(suffix);
                    Ok(Rc::clone(&content))
                }
                _ => Err(RuntimeError::MissmatchedTypes {
                    got: s.get_type(),
                    expected: ValueType::String,
                }),
            }
        }
    }
);

native_op!(
    StringPushLeft,
    ["string.<push", "string.push-left"],
    [s, content],
    {
        let prefix = content.expect_string()?; // must be a string

        let s_ptr = Rc::as_ptr(s) as *mut Value;

        unsafe {
            match &mut *s_ptr {
                Value::String(st) => {
                    // Efficient prepend without repeated shifting:
                    let suffix = std::mem::take(st);
                    let mut new = String::with_capacity(prefix.len() + suffix.len());
                    new.push_str(prefix);
                    new.push_str(&suffix);
                    *st = new;

                    Ok(Rc::clone(&content))
                }
                _ => Err(RuntimeError::MissmatchedTypes {
                    got: s.get_type(),
                    expected: ValueType::String,
                }),
            }
        }
    }
);

pub fn bind_string_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help("string.".to_string(), "string module:

- string: converts value to string
- string.split: splits by delimiter
- string.+: concatenates strings
- string.=: checks equality
- string.trim: removes whitespace from both ends
- string.trim-start: removes whitespace from start
- string.trim-end: removes whitespace from end
- string.upper: converts to uppercase
- string.lower: converts to lowercase
- string.has: checks if contains substring
- string.starts-with: checks prefix
- string.ends-with: checks suffix
- string.replace: replaces occurrences
- string.len: returns character count
- string.at: gets character at index
- string.slice: extracts substring
- string.index-of: finds substring index
- string.join: joins list with separator
- string.lines: splits into lines
- string.bytes: formats bytes as human-readable
- string.set: sets character at index (mutates)
- string.push, string.push>, string.push-right: appends (mutates)
- string.<push, string.push-left: prepends (mutates)
- string.iterate: iterates characters forward
- string.<iterate, string.iterate-backward: iterates backward".to_string());

    define_native!(MakeString, env, inter);
    define_help!(MakeString, env, "[value]: converts value to string\n\nstring 42  ; \"42\"");

    define_native!(StringSplit, env, inter);
    define_help!(StringSplit, env, "[delimiter:string string]: splits string by delimiter, returns list\n\nstring.split \",\" \"a,b,c\"  ; [\"a\" \"b\" \"c\"]");

    define_native!(StringConcat, env, inter);
    define_help!(StringConcat, env, "[string string]: concatenates two strings\n\nstring.+ \"hello\" \" world\"  ; \"hello world\"");

    define_native!(StringEq, env, inter);
    define_help!(StringEq, env, "[string string]: checks string equality\n\nstring.= \"abc\" \"abc\"  ; true");

    define_native!(StringTrim, env, inter);
    define_help!(StringTrim, env, "[string]: removes whitespace from both ends\n\nstring.trim \"  hello  \"  ; \"hello\"");

    define_native!(StringTrimStart, env, inter);
    define_help!(StringTrimStart, env, "[string]: removes whitespace from start\n\nstring.trim-start \"  hello\"  ; \"hello\"");

    define_native!(StringTrimEnd, env, inter);
    define_help!(StringTrimEnd, env, "[string]: removes whitespace from end\n\nstring.trim-end \"hello  \"  ; \"hello\"");

    define_native!(StringUppercase, env, inter);
    define_help!(StringUppercase, env, "[string]: converts to uppercase\n\nstring.upper \"hello\"  ; \"HELLO\"");

    define_native!(StringLowercase, env, inter);
    define_help!(StringLowercase, env, "[string]: converts to lowercase\n\nstring.lower \"HELLO\"  ; \"hello\"");

    define_native!(StringContains, env, inter);
    define_help!(StringContains, env, "[needle:string haystack:string]: checks if string contains substring\n\nstring.has \"ell\" \"hello\"  ; true");

    define_native!(StringStartsWith, env, inter);
    define_help!(StringStartsWith, env, "[prefix:string string]: checks if string starts with prefix\n\nstring.starts-with \"hel\" \"hello\"  ; true");

    define_native!(StringEndsWith, env, inter);
    define_help!(StringEndsWith, env, "[suffix:string string]: checks if string ends with suffix\n\nstring.ends-with \"lo\" \"hello\"  ; true");

    define_native!(StringReplace, env, inter);
    define_help!(StringReplace, env, "[from:string to:string string]: replaces all occurrences\n\nstring.replace \"l\" \"L\" \"hello\"  ; \"heLLo\"");

    define_native!(StringLength, env, inter);
    define_help!(StringLength, env, "[string]: returns character count\n\nstring.len \"hello\"  ; 5");

    define_native!(StringCharAt, env, inter);
    define_help!(StringCharAt, env, "[index:number string]: returns character at index, or null\n\nstring.at 0 \"hello\"  ; \"h\"");

    define_native!(StringSubstring, env, inter);
    define_help!(StringSubstring, env, "[start:number end:number string]: extracts substring from start to end index\n\nstring.slice 1 4 \"hello\"  ; \"ell\"");

    define_native!(StringIndexOf, env, inter);
    define_help!(StringIndexOf, env, "[needle:string haystack:string]: finds index of substring, -1 if not found\n\nstring.index-of \"ll\" \"hello\"  ; 2");

    define_native!(StringJoin, env, inter);
    define_help!(StringJoin, env, "[separator:string list]: joins list of strings with separator\n\nstring.join \", \" [\"a\" \"b\" \"c\"]  ; \"a, b, c\"");

    define_native!(StringLines, env, inter);
    define_help!(StringLines, env, "[string]: splits string into lines\n\nstring.lines \"a\\nb\\nc\"  ; [\"a\" \"b\" \"c\"]");

    define_native!(StringBytes, env, inter);
    define_help!(StringBytes, env, "[bytes:number]: formats bytes as human-readable string\n\nstring.bytes 1536  ; \"1.5 KiB\"");

    define_native!(StringSet, env, inter);
    define_help!(StringSet, env, "[index:number string replacement:string]: sets character at index (mutates string)\n\nstring.set 0 mystr \"H\"");

    define_native!(StringPush, env, inter);
    define_help!(StringPush, env, "[string suffix:string]: appends to end of string (mutates string)\n\nstring.push mystr \"!\"");

    define_native!(StringPushLeft, env, inter);
    define_help!(StringPushLeft, env, "[string prefix:string]: prepends to start of string (mutates string)\n\nstring.<push mystr \"Hello \"");

    define_native!(StringIterate, env, inter);
    define_help!(StringIterate, env, "[callback:lambda string]: calls function for each character\n\nstring.iterate print \"abc\"");

    define_native!(StringIterateBackward, env, inter);
    define_help!(StringIterateBackward, env, "[callback:lambda string]: iterates characters in reverse\n\nstring.<iterate print \"abc\"  ; prints c, b, a");
}
