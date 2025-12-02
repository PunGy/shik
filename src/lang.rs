//! Language interpretation module

use crate::parser::{parse, ParseResult, Program};

pub fn evaluate(input: &str) -> ParseResult<Program> {
    parse(input)
}

pub fn print(input: ParseResult<Program>) {
    match input {
        Ok(program) => {
            println!("  Parsed: {} statements", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("    Statement {}: {:?}", i, stmt.expression);
            }
        }
        Err(e) => println!("  Parse Error: {:?}", e),
    }
}

pub fn run_repl() {
    use std::io::{self, Write};

    println!("=== SHIK ===");
    println!("Enter expressions to evaluate, or 'quit' to exit.");
    println!("Type 'help' for available commands.\n");

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
            _ => print(evaluate(input)),
        }
    }
}

pub fn eval_file(path: String) {
    use std::fs::read_to_string;

    let contents = read_to_string(path).expect("Unable to open the file");

    print(evaluate(&contents))
}
