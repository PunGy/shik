use shik::parser::{Lexer, Parser};

fn main() {
    let examples = vec![
        ("pipe with app", "str $> map string.upper"),
        ("compose", "string.upper #> string.reverse"),
        (
            "full example",
            "\"hello\" $> map string.upper #> string.reverse",
        ),
        ("precedence test", "a #> b $> c"),
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
                        println!("  Parsed: {} statements", program.statements.len());
                        for (i, stmt) in program.statements.iter().enumerate() {
                            println!("    Statement {}: {:?}", i, stmt.expression);
                        }
                    }
                    Err(e) => println!("  Parse Error: {:?}", e),
                }
            }
            Err(e) => println!("  Lex Error: {:?}", e),
        }
    }
}

