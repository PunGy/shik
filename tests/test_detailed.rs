use shik::parser::{Lexer, Parser};

fn main() {
    let examples = vec![
        ("compose only", "f #> g"),
        ("app only", "f g"),
        ("app then compose", "f g #> h"),
        ("compose then app", "f #> g h"),
        ("map compose", "map f #> g"),
    ];

    for (name, input) in examples {
        println!("\n{}: {}", name, input);
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => {
                println!(
                    "  Tokens: {:?}",
                    tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(program) => {
                        for stmt in &program.statements {
                            print_expr(&stmt.expression, 2);
                        }
                    }
                    Err(e) => println!("  Parse Error: {:?}", e),
                }
            }
            Err(e) => println!("  Lex Error: {:?}", e),
        }
    }
}

fn print_expr(expr: &shik::parser::Expression, indent: usize) {
    let prefix = " ".repeat(indent);
    use shik::parser::Expression;
    match expr {
        Expression::Identifier(s) => println!("{}Ident: {}", prefix, s),
        Expression::Flow { left, right } => {
            println!("{}Flow:", prefix);
            print_expr(left, indent + 2);
            print_expr(right, indent + 2);
        }
        Expression::Pipe { left, right } => {
            println!("{}Pipe:", prefix);
            print_expr(left, indent + 2);
            print_expr(right, indent + 2);
        }
        Expression::Application { function, argument } => {
            println!("{}App:", prefix);
            print_expr(function, indent + 2);
            print_expr(argument, indent + 2);
        }
        _ => println!("{}{:?}", prefix, expr),
    }
}

