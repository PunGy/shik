use shik::parser::Lexer;

fn main() {
    let inputs = vec!["x $> f", "f #> g", "x $> f y"];

    for input in inputs {
        println!("\nInput: {}", input);
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => {
                for token in tokens {
                    println!("  {:?}", token);
                }
            }
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}

