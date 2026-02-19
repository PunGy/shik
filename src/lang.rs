//! Language interpretation module

use crate::eval::error::RuntimeError;
use crate::eval::evaluator::Interpretator;
use crate::eval::value::{Value, ValueRef};
use crate::parser::{parse, ParseError, Program};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Parsing failed: {0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
}

pub fn evaluate(input: &str, interpretator: &Interpretator) -> Result<ValueRef, EvalError> {
    let program = parse(input)?;
    let result = interpretator.interpretate(&program)?;

    Ok(result)
}

pub fn print(input: Result<ValueRef, EvalError>, silent: bool) {
    match input {
        Ok(res) => {
            if !silent {
                match res.as_ref() {
                    Value::String(str) => println!("\n⋖ \"{}\"", str),
                    _ => println!("\n⋖ {}", res),
                };
            }
        }
        Err(e) => println!("{}", e),
    }
}

pub fn print_ast(input: Result<Program, ParseError>) {
    match input {
        Ok(res) => {
            println!("{:?}", res);
        },
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
}


pub fn eval_file(path: String) {
    use std::fs::read_to_string;

    let contents = read_to_string(path).expect("Unable to open the file");
    let interpretator = Interpretator::new();

    print(evaluate(&contents, &interpretator), true)
}
