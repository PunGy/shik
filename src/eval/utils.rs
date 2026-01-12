use std::rc::Rc;

use crate::{
    eval::{
        error::RuntimeError,
        value::{EnvRef, MatchContext, Value, ValueRef},
        EvalResult,
    },
    parser::{ast::LiteralPattern, MatchPattern},
};

pub fn define_match(
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
                define_match(pattern, &inner_val, env, &match_context)?;
            }

            match rest {
                Some(rest_name) => {
                    // Use drop() to get a view of the remaining elements - O(1)
                    let rest_list = val_list.drop(last_inx + 1);
                    env.define(
                        rest_name.clone(),
                        Rc::new(Value::List(rest_list)),
                    );
                }
                _ => (),
            };
        }
        MatchPattern::Literal(_) => match match_context {
            MatchContext::Let => {
                return Err(RuntimeError::InvalidPatternMatching);
            }
            _ => (),
        },
        _ => (),
    }

    Ok(Rc::new(Value::Null))
}

pub fn pattern_match(pattern: &MatchPattern, item_val: &ValueRef, env: &EnvRef) -> Result<bool, RuntimeError> {
    match pattern {
        MatchPattern::Identifier(name) => {
            env.define(name.to_string(), Rc::clone(&item_val));
            return Ok(true)
        }
        MatchPattern::List { patterns, rest } => {
            let val_list;
            match item_val.as_ref() {
                Value::List(lst) => val_list = lst,
                _ => return Ok(false)
            }

            if *rest == None && patterns.len() != val_list.len() {
                return Ok(false);
            }

            let mut last_inx = 0;
            for (inx, pattern) in patterns.iter().enumerate() {
                last_inx = inx;
                let inner_val = val_list
                    .get(inx)
                    .ok_or_else(|| RuntimeError::InvalidPatternMatching)?;
                let success = pattern_match(pattern, &inner_val, env)?;
                if !success {
                    return Ok(false);
                }
            }

            match rest {
                Some(rest_name) => {
                    // Use drop() to get a view of the remaining elements - O(1)
                    let rest_list = val_list.drop(last_inx + 1);
                    env.define(
                        rest_name.clone(),
                        Rc::new(Value::List(rest_list)),
                    );
                }
                _ => (),
            };

            return Ok(true);
        }
        MatchPattern::Literal(lit) => {
            let is_eq = match (item_val.as_ref(), lit) {
                (Value::Number(item_num), LiteralPattern::Number(pattern_num)) => {
                    pattern_num == item_num
                }
                (Value::String(item_string), LiteralPattern::String(pattern_string)) => {
                    pattern_string == item_string
                }
                _ => false,
            };

            if is_eq {
                return Ok(true);
            }
        }
        MatchPattern::Wildcard => return Ok(true),
        _ => (),
    }

    Ok(false)
}
