//! Shared evaluation helpers for integration tests.
//!
//! Each integration test crate that needs them declares `mod common;`.

use shik::eval::evaluator::Interpretator;
use shik::eval::value::Value;
use shik::lang::evaluate;

/// Evaluate and get the result's display form.
#[allow(dead_code)]
pub fn eval(code: &str) -> Result<String, String> {
    let inter = Interpretator::new();
    match evaluate(code, &inter) {
        Ok(val) => Ok(val.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Evaluate and check if it's an error.
#[allow(dead_code)]
pub fn eval_is_error(code: &str) -> bool {
    let inter = Interpretator::new();
    evaluate(code, &inter).is_err()
}

/// Evaluate and get the number result.
#[allow(dead_code)]
pub fn eval_number(code: &str) -> Result<f64, String> {
    let inter = Interpretator::new();
    match evaluate(code, &inter) {
        Ok(val) => match val.as_ref() {
            Value::Number(n) => Ok(*n),
            _ => Err(format!("Expected number, got {}", val)),
        },
        Err(e) => Err(e.to_string()),
    }
}
