use shik::parser::Lexer;

fn main() {
    let inputs = vec!["\n\n s x\nf x"];

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

