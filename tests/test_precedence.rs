use shik::parser::{Lexer, Parser};

fn main() {
    let input = "a #> b $> c";
    println!("Input: {}", input);

    let mut lexer = Lexer::new(input);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!(
                "Tokens: {:?}",
                tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
            );
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    println!("Parsed: {} statements", program.statements.len());
                    for (i, stmt) in program.statements.iter().enumerate() {
                        println!("Statement {}: {:?}", i, stmt.expression);
                    }
                }
                Err(e) => println!("Parse Error: {:?}", e),
            }
        }
        Err(e) => println!("Lex Error: {:?}", e),
    }
}

