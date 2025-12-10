use crate::{
    count_args,
    eval::{
        error::{RuntimeError, ShikError},
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeContext, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
    define_native,
};
use glob::glob;
use std::fs;
use std::rc::Rc;

native_op!(FileRead, "file.read", [path], {
    let path = path.expect_string()?;

    let content = fs::read_to_string(path)
        .map_err(|e| ShikError::default_error(format!("Cannot open file: {}", e)))?;

    native_result(Value::String(content))
});

native_op!(FileTryRead, "file.try-read", [path], {
    let path = path.expect_string()?;

    match fs::read_to_string(path) {
        Ok(content) => native_result(Value::String(content)),
        Err(_) => native_result(Value::Null),
    }
});

native_op!(FileGlob, "file.glob", [pattern], {
    let pattern = pattern.expect_string()?;

    let paths = glob(pattern)
        .map_err(|e| ShikError::default_error(format!("invalid glob pattern: {}", e)))?;

    let mut result: Vec<ValueRef> = Vec::new();
    for entry in paths {
        match entry {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                result.push(Rc::new(Value::String(path_str)));
            }
            Err(e) => {
                return Err(ShikError::default_error(format!("glob error: {}", e)));
            }
        }
    }

    native_result(Value::List(result))
});

native_op!(FileWrite, "file.write", [path, content], {
    let path = path.expect_string()?;
    let content = content.expect_string()?;

    fs::write(path, content)
        .map_err(|e| ShikError::default_error(format!("cannot write file {}: {}", path, e)))?;

    native_result(Value::Null)
});

native_op!(FileAppend, "file.append", [path, content], {
    let path = path.expect_string()?;
    let content = content.expect_string()?;

    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ShikError::default_error(format!("cannot open file {}: {}", path, e)))?;

    file.write_all(content.as_bytes())
        .map_err(|e| ShikError::default_error(format!("cannot write to file {}: {}", path, e)))?;

    native_result(Value::Null)
});

native_op!(FileExists, "file.exists", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(std::path::Path::new(path).exists()))
});

native_op!(FileIsDir, "file.is-dir", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(std::path::Path::new(path).is_dir()))
});

native_op!(FileIsFile, "file.is-file", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(std::path::Path::new(path).is_file()))
});

pub fn bind_file_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(FileRead, env, inter);
    define_native!(FileTryRead, env, inter);
    define_native!(FileGlob, env, inter);
    define_native!(FileWrite, env, inter);
    define_native!(FileAppend, env, inter);
    define_native!(FileExists, env, inter);
    define_native!(FileIsDir, env, inter);
    define_native!(FileIsFile, env, inter);
}
