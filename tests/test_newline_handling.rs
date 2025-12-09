use shik::parser::{parse, Expression, Statement};

fn main() {
    println!("Testing newline handling in parser...\n");

    // Test 1: Multiple statements separated by newlines
    let input1 = "let x 10\nlet y 20\nx $> add y";
    println!("Test 1 - Multiple statements:");
    println!("Input:\n{}\n", input1);
    match parse(input1) {
        Ok(program) => {
            println!("Parsed {} statements:", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("  Statement {}: {:?}", i + 1, stmt.expression);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 2: Block with newlines (should group by line)
    let input2 = "'(\n  add 1 2\n  mul 3 4\n  sub 5 6\n)";
    println!("\nTest 2 - Block with newlines:");
    println!("Input:\n{}\n", input2);
    match parse(input2) {
        Ok(program) => {
            if let Some(stmt) = program.statements.first() {
                if let Expression::Block(exprs) = &stmt.expression {
                    println!("Block contains {} expressions:", exprs.len());
                    for (i, expr) in exprs.iter().enumerate() {
                        println!("  Line {}: {:?}", i + 1, expr);
                    }
                }
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 3: Lazy evaluation with newlines
    let input3 = "#(\n  x\n  y $> f\n  z\n)";
    println!("\nTest 3 - Lazy block with newlines:");
    println!("Input:\n{}\n", input3);
    match parse(input3) {
        Ok(program) => {
            if let Some(stmt) = program.statements.first() {
                if let Expression::Lazy(exprs) = &stmt.expression {
                    println!("Lazy block contains {} expressions:", exprs.len());
                    for (i, expr) in exprs.iter().enumerate() {
                        println!("  Line {}: {:?}", i + 1, expr);
                    }
                }
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 4: Empty lines should be ignored
    let input4 = "x\n\n\ny\n\nz";
    println!("\nTest 4 - Empty lines:");
    println!("Input:\n{}\n", input4);
    match parse(input4) {
        Ok(program) => {
            println!("Parsed {} statements:", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("  Statement {}: {:?}", i + 1, stmt.expression);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 5: Complex nested structure
    let input5 = "let process (fn [data] '(\n  let cleaned data $> trim\n  let validated cleaned $> validate\n  validated $> save\n))";
    println!("\nTest 5 - Complex nested structure:");
    println!("Input:\n{}\n", input5);
    match parse(input5) {
        Ok(program) => {
            println!("Successfully parsed complex structure");
            println!("Root: {:?}", program.statements[0].expression);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 6: Operators should not cross newlines
    let input6 = "x $>\nf\ny #>\ng";
    println!("\nTest 6 - Operators across newlines:");
    println!("Input:\n{}\n", input6);
    match parse(input6) {
        Ok(program) => {
            println!("Parsed {} statements:", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("  Statement {}: {:?}", i + 1, stmt.expression);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
