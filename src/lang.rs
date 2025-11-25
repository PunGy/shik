//! Language interpretation module

use crate::parser::{lexer::TokenizeResult, Lexer};

pub fn evaluate(input: &str) -> TokenizeResult {
    let mut lexer = Lexer::new(input);
    lexer.tokenize()
}

pub fn print(input: TokenizeResult) {
    match input {
        Ok(tokens) => {
            for token in tokens {
                println!("Token: [{:?}: ({})]", token.token_type, token.lexeme)
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
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
            _ => match evaluate(input) {
                Ok(tokens) => {
                    for token in tokens {
                        println!("Token: [{:?}: ({})]", token.token_type, token.lexeme)
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            },
        }
    }
}

pub fn eval_file(path: String) {
    use std::fs::read_to_string;

    let contents = read_to_string(path).expect("Unable to open the file");

    print(evaluate(&contents))
}
