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

#[derive(Error, Debug)]
pub enum FileError {
    #[error("shik: cannot open {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("{0}")]
    Eval(#[from] EvalError),
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
        Err(e) => eprintln!("{}", e),
    }
}

pub fn print_ast(input: Result<Program, ParseError>) {
    match input {
        Ok(res) => {
            println!("{:?}", res);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}

pub fn eval_file(path: &str, ast_mode: bool) -> Result<(), FileError> {
    use std::fs::read_to_string;

    let contents = read_to_string(path).map_err(|source| FileError::Io {
        path: path.to_string(),
        source,
    })?;

    if ast_mode {
        let program = parse(&contents).map_err(EvalError::from)?;
        println!("{:?}", program);
        return Ok(());
    }

    let interpretator = Interpretator::new();
    evaluate(&contents, &interpretator)?;
    Ok(())
}
