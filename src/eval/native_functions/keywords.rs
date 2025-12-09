use std::rc::Rc;

use crate::eval::value::{EnvRef, Value};

pub fn bind_keywords_module(env: &EnvRef) {
    // Yes, true and false are just variables, you can override them :D
    env.define("true".to_string(), Rc::new(Value::Bool(true)));
    env.define("false".to_string(), Rc::new(Value::Bool(false)));

    env.define("null".to_string(), Rc::new(Value::Null));
}
