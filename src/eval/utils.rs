use std::rc::Rc;

use crate::{
    eval::{
        error::RuntimeError, value::{EnvRef, MatchContext, Value, ValueRef}, EvalResult
    },
    parser::MatchPattern,
};

pub fn pattern_match(
    pattern: &MatchPattern,
    val: &ValueRef,
    env: &EnvRef,
    match_context: &MatchContext,
) -> EvalResult {
    match pattern {
        MatchPattern::Identifier(name) => {
            env.define(name.to_string(), Rc::clone(&val));
        }
        MatchPattern::List { patterns, rest } => {
            let val_list = val.expect_list()?;

            let mut last_inx = 0;
            for (inx, pattern) in patterns.iter().enumerate() {
                last_inx = inx;
                let inner_val = val_list
                    .get(inx)
                    .ok_or_else(|| RuntimeError::InvalidPatternMatching)?;
                pattern_match(pattern, inner_val, env, &match_context)?;
            }

            match rest {
                Some(rest_name) => env.define(
                    rest_name.clone(),
                    Rc::new(Value::List(
                        val_list[last_inx + 1..]
                            .iter()
                            .map(|v| Rc::clone(&v))
                            .collect(),
                    )),
                ),
                _ => (),
            };
        }
        MatchPattern::Literal(literal_pattern) => match match_context {
            MatchContext::Let => {
                return Err(RuntimeError::InvalidPatternMatching);
            }
            _ => (),
        },
        _ => (),
    }

    Ok(Rc::new(Value::Null))
}
