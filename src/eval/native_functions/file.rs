use crate::{
    count_args,
    eval::{
        error::{RuntimeError, ShikError},
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
use std::fs;
use std::rc::Rc;

native_op!(FileRead, "file.read", [path], {
    let path = path.expect_string()?;

    let content = fs::read_to_string(path)
        .map_err(|_| ShikError::default_error(format!("cannot open file: {}", path)))?;

    native_result(Value::String(content))
});

pub fn bind_file_module(env: &EnvRef) {
    FileRead::define(&env);
}
