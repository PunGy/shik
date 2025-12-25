//! Language interpretation module

use crate::eval::error::RuntimeError;
use crate::eval::evaluator::Interpretator;
use crate::eval::value::{Value, ValueRef};
use crate::parser::{parse, ParseError};
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
    // println!("parsing: {:#?}", program);
    let result = interpretator.interpretate(&program)?;

    Ok(result)
}

pub fn print(input: Result<ValueRef, EvalError>, silent: bool) {
    match input {
        Ok(res) => {
            if !silent {
                match res.as_ref() {
                    Value::String(str) => println!("\"{}\"", str),
                    _ => println!("{}", res),
                };
            }
        }
        Err(e) => println!("{}", e),
    }
}

pub fn run_repl() {
    use std::io::{self, Write};

    println!("=== SHIK ===");
    println!("Enter expressions to evaluate, or 'quit' to exit.");
    println!("Type 'help' for available commands.\n");

    let interpretator = Interpretator::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input");
            continue;
        }

        let input = input.trim();

        match input {
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            // "help" => {
            //     print_help();
            // }
            // "table" => {
            //     parser.print_table();
            // }
            "" => {
                // Empty input, just continue
            }
            _ => print(evaluate(input, &interpretator), false),
        }
    }
}

pub fn eval_file(path: String) {
    use std::fs::read_to_string;

    let contents = read_to_string(path).expect("Unable to open the file");
    let interpretator = Interpretator::new();

    print(evaluate(&contents, &interpretator), true)
}
